//! Compiler's prelude.

use std::collections::{HashMap, VecDeque};
use std::iter::FromIterator;
use std::ops::*;

use hir::def_id::DefId;
use linked_hash_map::LinkedHashMap;
use rustc_hir as hir;
use rustc_middle::ty::{
    AdtDef, GenericArgKind, GenericArgsRef, Generics, ParamEnv, Ty, TyCtxt, VariantDef, VariantDiscr,
};
use rustc_type_ir::TyKind;

use super::error::{VirgenError, VirgenResult};
use crate::utils::*;

/// Shape of an array.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shape {
    inner: VecDeque<usize>,

    /// Signedness.
    pub is_signed: bool,
}

impl Shape {
    /// Creates new shape.
    pub fn new<I: IntoIterator<Item = usize>>(iterable: I, is_signed: bool) -> Self {
        Self { inner: iterable.into_iter().collect(), is_signed }
    }

    /// Returns dimension of array.
    pub fn dim(&self) -> usize {
        self.inner.len()
    }

    /// Returns number of elements in array.
    pub fn width(&self) -> usize {
        self.inner.iter().product()
    }

    /// TODO: Documentation
    pub fn get(&self, index: usize) -> usize {
        assert!(self.dim() > index);
        *self.inner.get(index).unwrap()
    }

    /// TODO: Documentation
    #[must_use]
    pub fn multiple(&self, n: usize) -> Self {
        let mut inner = self.inner.clone();
        let front = inner.pop_front().unwrap();
        inner.push_front(front * n);

        Self { inner, is_signed: self.is_signed }
    }

    /// TODO: Documentation
    #[must_use]
    pub fn divide(&self, n: usize) -> Self {
        let mut inner = self.inner.clone();
        let front = inner.pop_front().unwrap();
        assert_eq!(front % n, 0);
        inner.push_front(front / n);

        Self { inner, is_signed: self.is_signed }
    }

    /// Returns signedness of the type.
    pub fn is_signed(&self) -> bool {
        self.is_signed
    }
}

/// Indicates how the discriminant of enum variants should be encoded.
#[derive(Debug, Clone, Default)]
pub enum EnumEncodingTy {
    /// Decimal encoding
    ///
    /// Each variant are encoded with decimal numbers, starting from 0.
    /// This will result in `clog2(N)` bits for the discriminant, where `N` is the number of variants.
    #[default]
    Decimal,

    /// One-hot encoding
    ///
    /// Each variant are encoded with one-hot encoding.
    /// This will result in `N` bits for the discriminant, where `N` is the number of variants.
    OneHot,

    /// TODO: Documentation
    ///
    /// NOTE: Reduces dynamic energy consumptiion
    Grey,
}

/// Type that handles enum variant's bit layout
#[derive(Debug, Clone)]
pub struct VariantLayout {
    /// Variant name
    name: String,
    /// Variant type
    typ: PortDecls,
    /// Encoded value.
    ///
    /// NOTE: If this is set for any of the variants, the enum encoding type should be `Decimal`
    /// for now.
    #[allow(unused)]
    discriminant: VariantDiscr,
}

/// Type that handles enum's bit layout
#[derive(Debug, Clone)]
pub enum AdtLayout {
    /// Enum type
    Enum {
        /// Enum name
        name: String,

        /// Variant discriminant encoding type
        encoding_ty: EnumEncodingTy,

        /// Variants
        variants: Vec<VariantLayout>,
    },

    /// Struct type
    Struct {
        /// Struct name
        name: String,

        /// Struct fields
        fields: Vec<(String, PortDecls)>,
    },

    /// Primitive array type
    Array {
        /// Element type
        elt_ty: PortDecls,

        /// Length of array
        len: usize,
    },
}

impl AdtLayout {
    /// Calculates bit layout of ADT.
    pub fn new<'tcx>(tcx: TyCtxt<'tcx>, def: &AdtDef<'tcx>, generic_args: GenericArgsRef<'tcx>) -> Self {
        let attr =
            def.did().as_local().and_then(|local| get_hazardflow_attribute(tcx, tcx.local_def_id_to_hir_id(local)));
        match def.adt_kind() {
            rustc_middle::ty::AdtKind::Struct => {
                if let Some(HazardFlowAttr::ExprMagic(ExprMagic::ArrayMagic(ArrayMagic::Array))) = attr {
                    let elt_ty = match generic_args.first().unwrap().unpack() {
                        GenericArgKind::Type(ty) => PortDecls::from_ty(ty, tcx).unwrap(),
                        _ => panic!(),
                    };

                    let len = evaluate_const_generic_arg(tcx, generic_args.get(1).unwrap()).unwrap();
                    return Self::Array { elt_ty, len };
                }

                assert!(def.variants().len() == 1);

                let struct_def = def.variant(0u32.into());

                Self::Struct {
                    name: struct_def.ident(tcx).to_string(),
                    fields: struct_def
                        .fields
                        .iter()
                        .map(|field| {
                            let ty = tcx.type_of(field.did).instantiate(tcx, generic_args);
                            let ty = normalize_alias_ty(tcx, ty);

                            (field.ident(tcx).to_string(), PortDecls::from_ty(ty, tcx).unwrap())
                        })
                        .collect(),
                }
            }
            rustc_middle::ty::AdtKind::Enum => {
                // TODO: check attrs to get discriminant encoding
                let encoding_ty = EnumEncodingTy::default();

                let variants = def
                    .variants()
                    .iter()
                    .map(|variant: &VariantDef| {
                        let name = variant.ident(tcx).to_string();
                        let typ = variant
                            .fields
                            .iter()
                            .map(|field| {
                                let ty = tcx.type_of(field.did).instantiate(tcx, generic_args);
                                let ty = normalize_alias_ty(tcx, ty);
                                (Some(field.ident(tcx).to_string()), PortDecls::from_ty(ty, tcx).unwrap())
                            })
                            .collect::<Vec<_>>();
                        VariantLayout { name, typ: PortDecls::Struct(typ), discriminant: variant.discr }
                    })
                    .collect();
                Self::Enum { name: tcx.item_name(def.did()).to_string(), encoding_ty, variants }
            }
            rustc_middle::ty::AdtKind::Union => todo!(),
        }
    }

    /// Returns bitwidth of the discriminant.
    pub fn discriminant_width(&self) -> usize {
        match self {
            AdtLayout::Enum { encoding_ty, variants, .. } => match encoding_ty {
                EnumEncodingTy::Decimal => clog2(variants.len()),
                EnumEncodingTy::OneHot => todo!(),
                EnumEncodingTy::Grey => todo!(),
            },
            AdtLayout::Struct { .. } => panic!(),
            AdtLayout::Array { .. } => panic!(),
        }
    }

    /// Returns type of the ADT.
    pub fn port_decls(&self) -> PortDecls {
        match self {
            AdtLayout::Enum { variants, .. } => PortDecls::Struct(
                [
                    vec![(Some("discriminant".to_string()), PortDecls::unsigned_bits(self.discriminant_width()))],
                    variants.iter().map(|variant| (Some(variant.name.clone()), variant.typ.clone())).collect(),
                ]
                .concat(),
            ),
            AdtLayout::Struct { fields, .. } => {
                PortDecls::Struct(fields.iter().map(|(name, field)| (Some(name.clone()), field.clone())).collect())
            }
            AdtLayout::Array { elt_ty, len } => elt_ty.multiple(*len),
        }
    }
}

/// Value type.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PortDecls {
    /// Collection of channels.
    Struct(Vec<(Option<String>, PortDecls)>),

    /// Single channel which contains its width.
    Bits(Shape),
}

impl std::fmt::Debug for PortDecls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Struct(strukt) => {
                if strukt.is_empty() {
                    return write!(f, "Unit");
                }

                let mut debug_builder = f.debug_struct("PortDecls");

                for (name, member) in strukt {
                    debug_builder.field(name.as_ref().unwrap_or(&"_".to_string()), member);
                }

                debug_builder.finish()
            }
            Self::Bits(shape) => match shape.dim() {
                0 => write!(f, "Unit"),
                1 => write!(f, "{}{}", if shape.is_signed() { "i" } else { "u" }, shape.get(0)),
                2 => write!(f, "{}{}x{}", if shape.is_signed() { "i" } else { "u" }, shape.get(0), shape.get(1)),
                _ => panic!(),
            },
        }
    }
}

impl PortDecls {
    /// TODO: Documentation
    pub fn shape(&self) -> Shape {
        match self {
            PortDecls::Struct(_) => todo!(),
            PortDecls::Bits(shape) => shape.clone(),
        }
    }

    /// Constructs unsigned bits type
    pub fn unsigned_bits(width: usize) -> Self {
        Self::Bits(Shape::new([width], false))
    }

    /// Constructs signed bits type
    pub fn signed_bits(width: usize) -> Self {
        Self::Bits(Shape::new([width], true))
    }

    /// Returns signedness of the PortDecls
    pub fn is_signed(&self) -> bool {
        match self {
            Self::Bits(shape) => shape.is_signed(),
            // XXX: because of enum
            _ => false,
        }
    }
}

impl PortDecls {
    /// TODO: remove all the unwraps
    pub fn from_ty<'tcx>(ty: Ty<'tcx>, tcx: TyCtxt<'tcx>) -> Option<Self> {
        match ty.kind() {
            TyKind::Bool => Some(Self::unsigned_bits(1)),
            TyKind::Int(int_ty) => {
                let width: usize = int_ty
                    .bit_width()
                    .expect("`isize` cannot be used as a signal type. Specify the bitwidth explicitly, such as `i32` or `i64`.")
                    .try_into()
                    .unwrap();

                Some(Self::signed_bits(width))
            }
            TyKind::Uint(uint_ty) => {
                let width = uint_ty
                    .bit_width()
                    .unwrap_or(32)
                    // .expect("`usize` cannot be used as a signal type. Specify the bitwidth explicitly, such as `u32` or `u64`.")
                    .try_into()
                    .unwrap();

                Some(Self::unsigned_bits(width))
            }
            TyKind::Adt(def, substs) => AdtLayout::new(tcx, def, substs).port_decls().into(),
            TyKind::Array(elt_ty, len) => {
                let c = len.eval_target_usize(tcx, ParamEnv::empty()) as usize;
                Some(Self::from_ty(*elt_ty, tcx)?.multiple(c))
            }
            TyKind::Tuple(ty) => {
                let inner = ty
                    .iter()
                    .enumerate()
                    .map(|(i, ty)| {
                        let ty = Self::from_ty(ty, tcx)?;

                        (Some(i.to_string()), ty).into()
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(Self::Struct(inner))
            }
            TyKind::Ref(r, t, m) => todo!("ref type {:#?} {:#?} {:#?}", r, t, m),
            unsupported_ty => {
                log::debug!(
                    "unsupported type conversion from rust Type {:#?} to PortDecls. You might need to normalize the type before the type conversion.",
                    unsupported_ty
                );
                None
            }
        }
    }

    /// Width of `PortDecls`.
    pub fn width(&self) -> usize {
        match self {
            PortDecls::Struct(inner) => inner.iter().map(|(_, m)| m.width()).sum(),
            PortDecls::Bits(shape) => shape.width(),
        }
    }

    /// Maximum dimension of the primitive value types in `PortDecls`.
    pub fn max_dim(&self) -> usize {
        self.iter().map(|(_, shape)| shape.dim()).max().unwrap_or(1)
    }

    /// Number of elements in `PortDecls`.
    pub fn num_elts(&self) -> usize {
        self.iter().count()
    }

    /// Iterator for `PortDecls`.
    ///
    /// # Note
    ///
    /// The iterator returns (name, width) for inner fields **ONLY** with nonzero width.
    /// This is to ignore meaningless unit types. (e.g. The unit type in `Keep<V, ()>`)
    pub fn iter(&self) -> ValueTypIterator {
        self.into_iter()
    }

    /// Iterator for `PortDecls`.
    ///
    /// XXX: This is a temporary method because we have to zip the iterator with zero-width ports.
    pub fn iter_with_zero_width(&self, prefix: Option<String>) -> ValueTypIterator {
        let mut iter_vec = vec![];

        match self {
            PortDecls::Struct(inner) => {
                for (name, member) in inner {
                    iter_vec
                        .extend(member.iter_with_zero_width(join_options("_", [prefix.clone(), name.clone()])).inner)
                }
            }
            PortDecls::Bits(shape) => {
                iter_vec.push((prefix, shape.clone()));
            }
        }

        ValueTypIterator { inner: iter_vec.into() }
    }

    /// Consumes the `PortDecls`, returning new `PortDecls` with width of each field multiplied by `n`.
    #[must_use]
    pub fn multiple(&self, n: usize) -> Self {
        match self {
            PortDecls::Struct(inner) => {
                PortDecls::Struct(inner.clone().into_iter().map(|(name, m)| (name, m.multiple(n))).collect::<Vec<_>>())
            }
            PortDecls::Bits(shape) => PortDecls::Bits(shape.multiple(n)),
        }
    }

    /// Consumes the `PortDecls`, returning new `PortDecls` with width of each field divided by `n`.
    #[must_use]
    pub fn divide(&self, n: usize) -> Self {
        match self {
            PortDecls::Struct(inner) => {
                PortDecls::Struct(inner.clone().into_iter().map(|(name, m)| (name, m.divide(n))).collect::<Vec<_>>())
            }
            PortDecls::Bits(shape) => PortDecls::Bits(shape.divide(n)),
        }
    }

    fn iter_with_prefix(&self, prefix: Option<String>) -> ValueTypIterator {
        let mut iter_vec = vec![];

        match self {
            PortDecls::Struct(inner) => {
                for (name, member) in inner {
                    iter_vec.extend(member.iter_with_prefix(join_options("_", [prefix.clone(), name.clone()])).inner)
                }
            }
            PortDecls::Bits(shape) => {
                if shape.width() > 0 {
                    iter_vec.push((prefix, shape.clone()));
                }
            }
        }

        ValueTypIterator { inner: iter_vec.into() }
    }
}

impl IntoIterator for &PortDecls {
    type IntoIter = ValueTypIterator;
    type Item = (Option<String>, Shape);

    fn into_iter(self) -> Self::IntoIter {
        self.iter_with_prefix(None)
    }
}

/// Iterator for `PortDecls`.
#[derive(Debug)]
pub struct ValueTypIterator {
    inner: VecDeque<(Option<String>, Shape)>,
}

impl Iterator for ValueTypIterator {
    type Item = (Option<String>, Shape);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_front()
    }
}

/// Channel's type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelTyp {
    /// Forward value.
    pub fwd: PortDecls,

    /// Backward value.
    pub bwd: PortDecls,
}

impl ChannelTyp {
    /// Creates a new channel type.
    pub const fn new(fwd: PortDecls, bwd: PortDecls) -> Self {
        Self { fwd, bwd }
    }
}

/// Interface's type.
#[allow(variant_size_differences)]
#[derive(Debug, Clone, Eq)]
pub enum InterfaceTyp {
    /// Unit type
    Unit,

    /// Single channel type
    Channel(ChannelTyp),

    /// Array of interface types
    Array(Box<InterfaceTyp>, usize),

    /// Struct of interface types. The first `String` of value indicates separator of the field.
    ///
    /// #[member(name="", sep = "" | nosep)]
    Struct(LinkedHashMap<String, (Option<String>, InterfaceTyp)>),
}

fn get_interface_impl<'tcx>(
    interface_ty: Ty<'tcx>,
    interface_trait_id: DefId,
    tcx: TyCtxt<'tcx>,
) -> VirgenResult<(hir::HirId, &'tcx hir::Impl<'tcx>)> {
    let mut impl_candidates = vec![];

    tcx.for_each_relevant_impl(interface_trait_id, interface_ty, |imp| {
        let hir_id = tcx.local_def_id_to_hir_id(imp.expect_local());

        let impl_item = tcx.hir().expect_item(imp.expect_local()).expect_impl();

        impl_candidates.push((hir_id, impl_item))
    });

    match impl_candidates.len() {
        1 => Ok(impl_candidates[0]),
        0 => Err(VirgenError::Misc { msg: format!("{:?} does not implement `Interface` trait", interface_ty) }),
        _ => Err(VirgenError::Misc { msg: format!("{:?} has multiple implementation of `Interface`", interface_ty) }),
    }
}

impl FromIterator<InterfaceTyp> for InterfaceTyp {
    fn from_iter<I: IntoIterator<Item = InterfaceTyp>>(iter: I) -> Self {
        let interfaces = iter.into_iter().collect::<Vec<_>>();
        match interfaces.len() {
            0 => Self::Unit,
            _ => {
                Self::Struct(interfaces.into_iter().enumerate().map(|(idx, i)| (idx.to_string(), (None, i))).collect())
            }
        }
    }
}

impl PartialEq for InterfaceTyp {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Channel(l0), Self::Channel(r0)) => l0 == r0,
            (Self::Array(l0, l1), Self::Array(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Struct(l0), Self::Struct(r0)) => {
                // NOTE: We manually compare the fields because `LinkedHashMap` compares the order of the fields.
                if l0.len() != r0.len() {
                    return false;
                }

                for (k, v) in l0 {
                    if let Some(r) = r0.get(k) {
                        if v != r {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                true
            }
            (Self::Unit, Self::Unit) => true,
            _ => false,
        }
    }
}

impl InterfaceTyp {
    /// Creates a new `InterfaceTyp` from `Ty`.
    pub fn from_ty<'tcx>(ty: Ty<'tcx>, interface_trait_id: DefId, tcx: TyCtxt<'tcx>) -> VirgenResult<Self> {
        // 1. Get relavent `Interface` implementation
        let (impl_id, interface_impl) = get_interface_impl(ty, interface_trait_id, tcx)?;

        match ty.kind() {
            rustc_type_ir::TyKind::Adt(e, substs) => {
                let e: &AdtDef<'tcx> = e;
                assert!(e.is_struct());

                let attribute = get_hazardflow_attribute(tcx, impl_id);
                if let Some(HazardFlowAttr::InterfaceMagic(InterfaceMagic::CompositeInterface)) = attribute {
                    // Composite interface
                    let fields = e
                        .all_fields()
                        .map(|field_def| {
                            let ty = field_def.ty(tcx, substs);
                            let interface_ty = Self::from_ty(ty, interface_trait_id, tcx).unwrap_or_else(|_| {
                                panic!("Composite interface should only have `Interface` as its field, but {:?} is not an `Interface`", tcx.def_ident_span(field_def.did))
                            });
                            (
                                field_def.name.to_ident_string(),
                                (None, interface_ty),
                            )
                        })
                        .collect();

                    return Ok(Self::Struct(fields));
                }

                // Primitive interface
                let impl_generics: &Generics = tcx.generics_of(impl_id.owner.def_id);

                assert_eq!(impl_generics.params.len(), substs.len());

                let assoc_items = interface_impl
                    .items
                    .iter()
                    .map(|item_ref| {
                        let name = item_ref.ident.to_string();

                        let item = tcx.hir().impl_item(item_ref.id);
                        let item = item.expect_type();

                        let typ_instantiated = tcx.type_of(item.hir_id.owner.def_id).instantiate(tcx, substs);

                        (name, typ_instantiated)
                    })
                    // .map(|(name, ty)| (name, tcx.type_of(ty.hir_id.owner.def_id).subst(tcx, substs)))
                    .collect::<Vec<_>>();
                let fwd = assoc_items
                    .iter()
                    .find(|(name, _)| name == "Fwd")
                    .map(|(_, x)| normalize_alias_ty(tcx, *x))
                    .and_then(|ty| PortDecls::from_ty(ty, tcx))
                    .ok_or_else(|| VirgenError::Misc { msg: "Interface impl does not have `Fwd`".to_string() })?;
                let bwd = assoc_items
                    .iter()
                    .find(|(name, _)| name == "Bwd")
                    .map(|(_, x)| normalize_alias_ty(tcx, *x))
                    .and_then(|ty| PortDecls::from_ty(ty, tcx))
                    .ok_or_else(|| VirgenError::Misc { msg: "Interface impl does not have `Bwd`".to_string() })?;

                Ok(InterfaceTyp::Channel(ChannelTyp { fwd, bwd }))
            }
            rustc_type_ir::TyKind::Tuple(ty_list) => {
                let mut interface_map = LinkedHashMap::new();

                for (i, ty) in ty_list.iter().enumerate() {
                    let interface_ty = Self::from_ty(ty, interface_trait_id, tcx)?;

                    assert_eq!(
                        // TODO: Use `sep` attribute
                        interface_map.insert(i.to_string(), (None, interface_ty)),
                        None,
                        "Interface type should not have duplicated index"
                    );
                }

                Ok(if interface_map.is_empty() { Self::Unit } else { Self::Struct(interface_map) })
            }
            rustc_type_ir::TyKind::Array(elt_ty, len) => {
                let elt_ty = Self::from_ty(*elt_ty, interface_trait_id, tcx)?;
                Ok(Self::Array(Box::new(elt_ty), len.eval_target_usize(tcx, ParamEnv::empty()).try_into().unwrap()))
            }
            _ => todo!("not implemented {:#?}", ty),
        }
    }

    /// Returns channel_typ if the interface type is single channel.
    pub fn get_channel_typ(self) -> Option<ChannelTyp> {
        if let InterfaceTyp::Channel(channel_typ) = self {
            Some(channel_typ)
        } else {
            None
        }
    }

    /// Returns primitive interface types and their endpoint paths in the interface type.
    #[allow(clippy::wrong_self_convention)]
    pub fn into_primitives(&self) -> Vec<(InterfaceTyp, EndpointPath)> {
        match self {
            InterfaceTyp::Unit | InterfaceTyp::Channel(_) => {
                vec![(self.clone(), EndpointPath::default())]
            }
            InterfaceTyp::Array(interface_typ, count) => (0..*count)
                .flat_map(|i| {
                    interface_typ.into_primitives().into_iter().map(move |(primitive_typ, mut path)| {
                        path.inner.push_front(EndpointNode::Index(i));
                        (primitive_typ, path)
                    })
                })
                .collect(),
            InterfaceTyp::Struct(inner) => inner
                .into_iter()
                .flat_map(|(name, (sep, interface_typ))| {
                    interface_typ.into_primitives().into_iter().map(|(primitive_typ, mut path)| {
                        path.inner.push_front(EndpointNode::Field(name.clone(), sep.clone()));
                        (primitive_typ, path)
                    })
                })
                .collect(),
        }
    }

    /// Returns subinterface given a endpoint path
    pub fn get_subinterface(&self, mut path: EndpointPath) -> Self {
        if let Some(front) = path.pop_front() {
            match (front, self) {
                (EndpointNode::Index(i), InterfaceTyp::Array(typ, size)) => {
                    assert!(i < *size);
                    typ.get_subinterface(path)
                }
                (EndpointNode::Field(field, _), InterfaceTyp::Struct(map)) => {
                    if let Some((_, typ)) = map.get(&field) {
                        typ.get_subinterface(path)
                    } else {
                        panic!("{field} does not exist in the struct")
                    }
                }
                _ => panic!("path and interface doesn't match"),
            }
        } else {
            self.clone()
        }
    }

    /// Returns true if the interface contains channel.
    pub fn contains_channel(&self) -> bool {
        self.into_primitives().into_iter().any(|(interface_typ, _)| interface_typ.get_channel_typ().is_some())
    }

    /// Computes the product of array lengths recursively
    ///
    /// Returns the product of all array lengths encountered during the traversal. If `self` is not `Array`,
    /// it contributes with a length of 1 to the product.
    ///
    /// Used to identify the bit length of a certain value.
    /// For more information, please refer to the `gen_module_seq_assigns` function in `codegen.rs`.
    pub fn nested_array_flattened_len(&self) -> usize {
        match self {
            InterfaceTyp::Array(inner, len) => inner.nested_array_flattened_len() * len,
            _ => 1,
        }
    }
}

/// Input/output channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    /// Channel's typ.
    pub typ: ChannelTyp,

    /// Channel's endpoint.
    pub endpoint: Endpoint,
}

impl Channel {
    /// Returns channel type.
    pub fn typ(&self) -> ChannelTyp {
        self.typ.clone()
    }

    /// Returns endpoint.
    pub fn endpoint(&self) -> Endpoint {
        self.endpoint.clone()
    }
}

/// Input/output interface.
#[allow(variant_size_differences)]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Interface {
    /// Unit
    #[default]
    Unit,

    /// Single channel
    Channel(Channel),

    /// Array of interfaces
    Array(Vec<Interface>),

    /// Struct of interfaces. The first `Option<String>` of value indicates separator of the field.
    /// If it is `None`, then separator is '_'.
    Struct(LinkedHashMap<String, (Option<String>, Interface)>),

    /// Unwired interface
    ///
    /// TODO: Documentation
    Unwired(InterfaceTyp),
}

impl Interface {
    /// Returns the channel if the interface is channel.
    pub fn get_channel(self) -> Option<Channel> {
        if let Interface::Channel(channel) = self {
            Some(channel)
        } else {
            None
        }
    }

    /// Returns the interface type.
    pub fn typ(&self) -> InterfaceTyp {
        match self {
            Interface::Unit => InterfaceTyp::Unit,
            Interface::Channel(channel) => InterfaceTyp::Channel(channel.typ.clone()),
            Interface::Array(inner) => InterfaceTyp::Array(Box::new(inner[0].typ()), inner.len()),
            Interface::Struct(inner) => InterfaceTyp::Struct(
                inner.iter().map(|(name, (sep, interface))| (name.clone(), (sep.clone(), interface.typ()))).collect(),
            ),
            Interface::Unwired(inner) => inner.clone(),
        }
    }

    /// Returns primitive interfaces in the interface.
    #[allow(clippy::wrong_self_convention)]
    pub fn into_primitives(&self) -> Vec<(Interface, EndpointPath)> {
        match self {
            Interface::Unit => vec![],
            Interface::Channel(_) => {
                vec![(self.clone(), EndpointPath::default())]
            }
            Interface::Array(interfaces) => interfaces
                .iter()
                .enumerate()
                .flat_map(|(i, interface)| {
                    interface.into_primitives().into_iter().map(move |(primitive, mut path)| {
                        path.inner.push_front(EndpointNode::Index(i));
                        (primitive, path)
                    })
                })
                .collect(),
            Interface::Struct(inner) => inner
                .iter()
                .flat_map(|(name, (sep, interface))| {
                    interface.into_primitives().into_iter().map(|(primitive, mut path)| {
                        path.inner.push_front(EndpointNode::Field(name.clone(), sep.clone()));
                        (primitive, path)
                    })
                })
                .collect(),
            Interface::Unwired(typ) => panic!("found unwired interface {typ:?}"),
        }
    }

    /// Returns subinterface given a endpoint path
    pub fn get_subinterface(&self, mut path: EndpointPath) -> Self {
        if let Some(node) = path.pop_front() {
            match (node, self) {
                (EndpointNode::Field(field, _), Self::Struct(map)) => {
                    if let Some((_, typ)) = map.get(&field) {
                        typ.get_subinterface(path)
                    } else {
                        panic!("{field} does not exist in the struct")
                    }
                }
                (EndpointNode::Index(idx), Self::Array(inner)) => {
                    assert!(idx < inner.len());
                    inner[idx].get_subinterface(path)
                }
                (node, interface) => panic!("{:?} and {:?} doesn't match", node, interface),
            }
        } else {
            self.clone()
        }
    }

    /// Returns itself with the
    pub fn swap_field(self, field: &str, swap: Self) -> Self {
        if let Self::Struct(mut map) = self {
            map.entry(field.to_string()).and_modify(|entry| entry.1 = swap);
            Self::Struct(map)
        } else {
            panic!("`swap_field` is only available for `Struct`")
        }
    }

    /// Wire the interface given a endpoint path
    /// Wire the `&mut self` interface with the given interface `interface` at the given endpoint path `path`.
    pub fn wire(&mut self, mut path: EndpointPath, interface: Interface) {
        assert!(!matches!(self, Interface::Channel(_)), "`self` shouldn't be already wired interface.\n{self:#?}");
        if let Some(node) = path.pop_front() {
            // Partially wire to the designated path

            assert!(matches!(self, Interface::Unwired(_) | Interface::Array(_) | Interface::Struct(_)), "{self:#?}");
            if let Self::Unwired(inner) = self {
                match inner {
                    InterfaceTyp::Array(inner, len) => {
                        *self = Self::Array(
                            (0..*len)
                                .map(|_| match inner.as_ref() {
                                    InterfaceTyp::Unit => Self::Unit,
                                    _ => Self::Unwired(*inner.clone()),
                                })
                                .collect(),
                        );
                    }
                    InterfaceTyp::Struct(inner) => {
                        *self = Self::Struct(
                            inner
                                .into_iter()
                                .map(|(field, (sep, typ))| {
                                    (
                                        field.clone(),
                                        (sep.clone(), match typ {
                                            InterfaceTyp::Unit => Self::Unit,
                                            _ => Self::Unwired(typ.clone()),
                                        }),
                                    )
                                })
                                .collect(),
                        );
                    }
                    _ => todo!(),
                }
            }

            match (self, node) {
                (Interface::Array(inner), EndpointNode::Index(i)) => inner[i].wire(path, interface),
                (Interface::Struct(inner), EndpointNode::Field(field, _)) => {
                    if let Some(field) = inner.get_mut(&field) {
                        field.1.wire(path, interface)
                    } else {
                        panic!("failed to wire {interface:?}:{field} does not exist in the struct {inner:?}")
                    }
                }
                _ => panic!(),
            }
        } else {
            // Fully wire the interface
            assert_eq!(self.typ(), interface.typ());
            *self = interface;
        }
    }

    /// Returns true if the interface contains unwired interface.
    pub fn contains_unwired(&self) -> bool {
        match self {
            Interface::Unwired(_) => true,
            Interface::Array(inner) => inner.iter().any(|i| i.contains_unwired()),
            Interface::Struct(inner) => inner.values().any(|(_, i)| i.contains_unwired()),
            _ => false,
        }
    }
}

impl FromIterator<Interface> for Interface {
    fn from_iter<I: IntoIterator<Item = Interface>>(iter: I) -> Self {
        let interfaces = iter.into_iter().collect::<Vec<_>>();
        match interfaces.len() {
            0 => Self::Unit,
            _ => {
                Self::Struct(interfaces.into_iter().enumerate().map(|(idx, i)| (idx.to_string(), (None, i))).collect())
            }
        }
    }
}

impl FromIterator<(Interface, EndpointPath)> for Interface {
    /// Constructs interface from primitive interfaces.
    fn from_iter<I: IntoIterator<Item = (Interface, EndpointPath)>>(iter: I) -> Self {
        let mut primitives = iter.into_iter().collect::<Vec<_>>();
        assert!(!primitives.is_empty());

        let is_primitive = primitives[0].1.inner.front().is_none();
        if is_primitive {
            assert_eq!(primitives.len(), 1);
            let (primitive, _) = primitives.pop().unwrap();
            assert!(matches!(primitive, Interface::Unit | Interface::Channel(_)));
            primitive
        } else {
            match primitives[0].1.inner.front().unwrap() {
                EndpointNode::Index(_) => {
                    let mut interfaces = HashMap::<usize, Vec<(Interface, EndpointPath)>>::new();
                    for (interface, mut path) in primitives {
                        let node = path.inner.pop_front().unwrap();
                        match node {
                            EndpointNode::Index(i) => {
                                interfaces.entry(i).or_default();
                                let primitives = interfaces.get_mut(&i).unwrap();
                                primitives.push((interface, path));
                            }
                            _ => panic!("internal compiler error"),
                        }
                    }
                    let len = interfaces.len();
                    Interface::Array(
                        (0..len).map(|i| interfaces.get(&i).unwrap().clone().into_iter().collect()).collect(),
                    )
                }
                EndpointNode::Field(..) => {
                    let mut inner = LinkedHashMap::<String, (Option<String>, Vec<(Interface, EndpointPath)>)>::new();
                    for (interface, mut path) in primitives {
                        let node = path.inner.pop_front().unwrap();
                        match node {
                            EndpointNode::Field(name, sep) => {
                                inner.entry(name.clone()).or_insert((sep, Vec::new()));
                                let primitives = inner.get_mut(&name).unwrap();
                                primitives.1.push((interface, path));
                            }
                            _ => panic!("internal compiler error"),
                        }
                    }
                    Interface::Struct(
                        inner
                            .into_iter()
                            .map(|(name, (sep, primitives))| (name, (sep, primitives.into_iter().collect())))
                            .collect(),
                    )
                }
            }
        }
    }
}

/// Endpoint's node.
// TODO: Add array range types
#[allow(variant_size_differences)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EndpointNode {
    /// Element of array.
    Index(usize),

    /// Field of struct. The first `String` indicates name of the field, and the second `Option<String>`
    /// indicates separator. If it is `None`, then separator is '_'.
    Field(String, Option<String>),
}

/// Endpoint's path.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct EndpointPath {
    /// List of endpoint nodes.
    pub inner: VecDeque<EndpointNode>,
}

impl EndpointPath {
    /// Append other endpoint path to the path.
    pub fn append_path(&self, other: &EndpointPath) -> Self {
        let mut inner = self.inner.clone();
        inner.extend(other.inner.iter().cloned());
        Self { inner }
    }

    /// Append node to endpoint path.
    pub fn append_node(&self, other: EndpointNode) -> Self {
        let mut inner = self.inner.clone();
        inner.push_back(other);
        Self { inner }
    }

    /// Append `EndpointNode::Field` to the path.
    pub fn append_field_with_sep(&self, field: &str, sep: Option<String>) -> Self {
        self.append_node(EndpointNode::Field(field.to_string(), sep))
    }

    /// Append `EndpointNode::Field` to the path with default separator.
    pub fn append_field(&self, field: &str) -> Self {
        self.append_field_with_sep(field, None)
    }

    /// Append `EndpointNode::Index` to the path.
    pub fn append_index(&self, index: usize) -> Self {
        self.append_node(EndpointNode::Index(index))
    }
}

impl FromIterator<EndpointNode> for EndpointPath {
    fn from_iter<T: IntoIterator<Item = EndpointNode>>(iter: T) -> Self {
        Self { inner: iter.into_iter().collect() }
    }
}

impl Deref for EndpointPath {
    type Target = VecDeque<EndpointNode>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for EndpointPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Wire's endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    /// Input interface.
    Input {
        /// Interface's endpoint path in the input.
        path: EndpointPath,
    },

    /// Submodule endpoint.
    Submodule {
        /// Submodule's index in the module's submodules.
        submodule_index: usize,

        /// Interface's endpoint path in the submodule.
        path: EndpointPath,
    },
}

impl Endpoint {
    /// Creates a new endpoint on input.
    pub fn input(path: EndpointPath) -> Self {
        Self::Input { path }
    }

    /// Creates a new endpoint on submodule.
    pub fn submodule(submodule_index: usize, path: EndpointPath) -> Self {
        Self::Submodule { submodule_index, path }
    }

    /// Returns endpoint path.
    pub fn path(&self) -> &EndpointPath {
        match self {
            Endpoint::Input { path } => path,
            Endpoint::Submodule { path, .. } => path,
        }
    }
}

/// Unary operators.
// TODO: Add more cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    /// Negation
    Negation,
}

impl ToString for UnaryOp {
    fn to_string(&self) -> String {
        match self {
            UnaryOp::Negation => "~",
        }
        .to_string()
    }
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    /// Addition
    Add,

    /// Subtraction
    Sub,

    /// Multiplication
    Mul,

    /// Division
    Div,

    /// Modulus
    Mod,

    /// Or (bitwise)
    Or,

    /// And (bitwise)
    And,

    /// Xor (bitwise)
    Xor,

    /// Eq (bitwise, `a ~^ b`)
    Eq,

    /// Eq (arithmetic, `a == b`)
    EqArithmetic,

    /// Ne (arithmetic, strict, `a === b`)
    NeStrict,

    /// Ne (arithmetic, `a != b`)
    NeArithmetic,

    /// Less than
    Less,

    /// Greater than
    Greater,

    /// Less than or equal
    LessEq,

    /// Greater than or equal
    GreaterEq,

    /// Shift left
    ShiftLeft,

    /// Shift right
    ShiftRight,
}

impl From<rustc_middle::mir::BinOp> for BinaryOp {
    fn from(op: rustc_middle::mir::BinOp) -> Self {
        match op {
            rustc_middle::mir::BinOp::Add => BinaryOp::Add,
            rustc_middle::mir::BinOp::Mul => BinaryOp::Mul,
            rustc_middle::mir::BinOp::Sub => BinaryOp::Sub,
            rustc_middle::mir::BinOp::Div => BinaryOp::Div,
            rustc_middle::mir::BinOp::Rem => BinaryOp::Mod,
            rustc_middle::mir::BinOp::BitXor => BinaryOp::Xor,
            rustc_middle::mir::BinOp::BitAnd => BinaryOp::And,
            rustc_middle::mir::BinOp::BitOr => BinaryOp::Or,
            rustc_middle::mir::BinOp::Shl => BinaryOp::ShiftLeft,
            rustc_middle::mir::BinOp::Shr => BinaryOp::ShiftRight,
            rustc_middle::mir::BinOp::Eq => BinaryOp::EqArithmetic,
            rustc_middle::mir::BinOp::Lt => BinaryOp::Less,
            rustc_middle::mir::BinOp::Le => BinaryOp::LessEq,
            rustc_middle::mir::BinOp::Ne => BinaryOp::NeArithmetic,
            rustc_middle::mir::BinOp::Ge => BinaryOp::GreaterEq,
            rustc_middle::mir::BinOp::Gt => BinaryOp::Greater,
            rustc_middle::mir::BinOp::Offset => todo!(),
            rustc_middle::mir::BinOp::AddUnchecked => todo!(),
            rustc_middle::mir::BinOp::SubUnchecked => todo!(),
            rustc_middle::mir::BinOp::MulUnchecked => todo!(),
            rustc_middle::mir::BinOp::ShlUnchecked => todo!(),
            rustc_middle::mir::BinOp::ShrUnchecked => todo!(),
        }
    }
}

impl ToString for BinaryOp {
    fn to_string(&self) -> String {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Or => "|",
            BinaryOp::And => "&",
            BinaryOp::Xor => "^",
            BinaryOp::Eq => "~^",
            BinaryOp::EqArithmetic => "==",
            BinaryOp::Less => "<",
            BinaryOp::Greater => ">",
            BinaryOp::LessEq => "<=",
            BinaryOp::GreaterEq => ">=",
            BinaryOp::ShiftLeft => "<<",
            BinaryOp::ShiftRight => ">>>",
            BinaryOp::NeArithmetic => "!=",
            BinaryOp::NeStrict => "!==",
        }
        .to_string()
    }
}
