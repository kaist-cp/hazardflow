//! Hazardflow Attribute related utilities.

use rustc_ast::ast;
use rustc_hir as hir;
use rustc_middle::ty::TyCtxt;

use crate::compiler::BinaryOp;

/// Hazardflow Attributes, defined in `hazardflow-macros`
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(variant_size_differences)]
pub enum HazardFlowAttr {
    /// Synthesizable function
    Synthesize,

    /// Expression Magic.
    ExprMagic(ExprMagic),

    /// FFI.
    FFI {
        /// FFI Module Name.
        module_name: Box<String>,

        /// Module params
        params: Vec<String>,
    },

    /// Interface Magic.
    InterfaceMagic(InterfaceMagic),

    /// System Task.
    SystemTask(SystemTaskMagic),

    /// Module Magic.
    ModuleMagic(ModuleMagic),
}

/// Expression Magic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprMagic {
    /// Array Magic
    ArrayMagic(ArrayMagic),
    /// Int Magic
    IntMagic(IntMagic),
    /// Adt Magic
    AdtMagic(AdtMagic),
    /// X
    X,
}

impl HazardFlowAttr {
    fn array_magic(s: &str) -> Self {
        let magic = match s {
            "array" => ArrayMagic::Array,
            "set" => ArrayMagic::Set,
            "clip_const" => ArrayMagic::ClipConst,
            "zip" => ArrayMagic::Zip,
            "map" => ArrayMagic::Map,
            "fold" => ArrayMagic::Fold,
            "resize" => ArrayMagic::Resize,
            "chunk" => ArrayMagic::Chunk,
            "append" => ArrayMagic::Append,
            "concat" => ArrayMagic::Concat,
            "range" => ArrayMagic::Range,
            "from" => ArrayMagic::From,
            "index" => ArrayMagic::Index,
            "bitor" => ArrayMagic::BitOr,
            "bitand" => ArrayMagic::BitAnd,
            "bitxor" => ArrayMagic::BitXor,
            "repeat" => ArrayMagic::Repeat,
            "eq" => ArrayMagic::Eq,
            "ne" => ArrayMagic::Ne,
            "set_range" => ArrayMagic::SetRange,
            _ => panic!("Invalid Magic, register it. {:?}", s),
        };

        HazardFlowAttr::ExprMagic(ExprMagic::ArrayMagic(magic))
    }

    fn interface_magic(s: &str) -> Self {
        match s {
            "fsm" => HazardFlowAttr::InterfaceMagic(InterfaceMagic::Fsm),
            "composite_interface" => HazardFlowAttr::InterfaceMagic(InterfaceMagic::CompositeInterface),
            _ => panic!("Invalid Magic, register it. {:?}", s),
        }
    }

    fn int_magic(s: &str) -> Self {
        let magic = match s {
            "lt" => IntMagic::Lt,
            "le" => IntMagic::Le,
            "gt" => IntMagic::Gt,
            "ge" => IntMagic::Ge,
            "convert" => IntMagic::Convert,
            "sub" => IntMagic::Sub,
            "add" => IntMagic::Add,
            "shl" => IntMagic::Shl,
            "shr" => IntMagic::Shr,
            "not" => IntMagic::Not,
            "mul" => IntMagic::Mul,
            _ => panic!("Invalid Magic, register it. {:?}", s),
        };

        HazardFlowAttr::ExprMagic(ExprMagic::IntMagic(magic))
    }

    fn adt_magic(s: &str) -> Self {
        let magic = match s {
            "enum_eq" => AdtMagic::EnumEq,
            "enum_ne" => AdtMagic::EnumNe,
            _ => panic!("Invalid Magic, register it. {:?}", s),
        };

        HazardFlowAttr::ExprMagic(ExprMagic::AdtMagic(magic))
    }

    fn ffi(s: &str) -> HazardFlowAttr {
        let (module_name, params) = {
            let (module_name, rest) =
                s.split_once('(').unwrap_or_else(|| panic!("Wrong format for declaring ffi module: {:?}", s));
            let (params, _) =
                rest.split_once(')').unwrap_or_else(|| panic!("Wrong format for declaring ffi module: {:?}", rest));
            let params = params
                .split(',')
                .filter_map(|s| match s.trim() {
                    "" => None,
                    s => Some(s.to_string()),
                })
                .collect();
            (module_name.to_string().into(), params)
        };

        HazardFlowAttr::FFI { module_name, params }
    }

    fn system(s: &str) -> HazardFlowAttr {
        match s {
            "display" => HazardFlowAttr::SystemTask(SystemTaskMagic::Display),
            "assert" => HazardFlowAttr::SystemTask(SystemTaskMagic::Assert),
            _ => panic!("Invalid System Task, register it. {:?}", s),
        }
    }

    fn module_magic(s: &str) -> HazardFlowAttr {
        match s {
            "split" => HazardFlowAttr::ModuleMagic(ModuleMagic::ModuleSplit),
            "seq" => HazardFlowAttr::ModuleMagic(ModuleMagic::Seq),
            "from_fn" => HazardFlowAttr::ModuleMagic(ModuleMagic::FromFn),
            _ => panic!("Invalid Magic, register it. {:?}", s),
        }
    }
}

/// Interface Magics
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceMagic {
    /// Fsm
    Fsm,

    /// Composite Interface
    CompositeInterface,
}

/// Array Magics
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayMagic {
    /// Array
    Array,

    /// Set
    Set,

    /// Clip const
    ClipConst,

    /// Zip
    Zip,

    /// Map
    Map,

    /// Fold
    Fold,

    /// Resize
    Resize,

    /// Chunk
    Chunk,

    /// Append
    Append,

    /// Concat
    Concat,

    /// Range
    Range,

    /// From
    From,

    /// Index
    Index,

    /// BitOr
    BitOr,

    /// BitAnd
    BitAnd,

    /// Repeat
    Repeat,

    /// Eq
    Eq,

    /// Ne
    Ne,

    /// BitXor
    BitXor,

    /// Set Range
    SetRange,
}

/// Integer Magics
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntMagic {
    /// Not
    Not,

    /// Shr
    Shr,

    /// Shl
    Shl,

    /// Add
    Add,

    /// Sub
    Sub,

    /// Lt
    Lt,

    /// Le
    Le,

    /// Gt
    Gt,

    /// Ge
    Ge,

    /// Convert
    Convert,

    /// Mult
    Mul,
}

impl IntMagic {
    /// Returns BinaryOp according to the magic function.
    ///
    /// This panics if it is not a binary operation.
    pub fn bin_op(&self) -> BinaryOp {
        match self {
            IntMagic::Shr => BinaryOp::ShiftRight,
            IntMagic::Shl => BinaryOp::ShiftLeft,
            IntMagic::Add => BinaryOp::Add,
            IntMagic::Sub => BinaryOp::Sub,
            IntMagic::Lt => BinaryOp::Less,
            IntMagic::Le => BinaryOp::LessEq,
            IntMagic::Gt => BinaryOp::Greater,
            IntMagic::Ge => BinaryOp::GreaterEq,
            IntMagic::Mul => BinaryOp::Mul,
            IntMagic::Not => todo!(),
            IntMagic::Convert => todo!(),
        }
    }
}

/// Adt Magics
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdtMagic {
    /// Eq
    EnumEq,

    /// Ne
    EnumNe,
}

/// System Tasks
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemTaskMagic {
    /// Display
    Display,

    /// Display
    Assert,
}

/// Module Magic
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleMagic {
    /// Module Split.
    ModuleSplit,

    /// From Fn
    FromFn,

    /// Seq
    Seq,
}

/// Get Hazardflow Attributes attached to an item.
pub fn get_hazardflow_attribute(tcx: TyCtxt<'_>, hir_id: hir::HirId) -> Option<HazardFlowAttr> {
    let attrs = tcx
        .hir()
        .attrs(hir_id)
        .iter()
        .filter_map(|attr| -> Option<HazardFlowAttr> {
            match &attr.kind {
                ast::AttrKind::Normal(normal_attr) => {
                    let ast::AttrItem { path: ast::Path { segments, .. }, args, .. } = &normal_attr.item;

                    if segments.len() >= 2 && segments[0].ident.as_str() == "hazardflow" {
                        match segments[1].ident.as_str() {
                            "synthesize" => Some(HazardFlowAttr::Synthesize),
                            "magic" => match args {
                                rustc_ast::AttrArgs::Delimited(inner) => {
                                    let magic_name = inner.tokens.trees().next().unwrap();
                                    match magic_name {
                                        rustc_ast::tokenstream::TokenTree::Token(t, s) => {
                                            assert_eq!(s, &rustc_ast::tokenstream::Spacing::Alone);
                                            match t.kind {
                                                rustc_ast::token::TokenKind::Literal(l) => {
                                                    // HACK: Don't know why rust is giving newline
                                                    // as two characters..
                                                    let arg = l
                                                        .symbol
                                                        .to_ident_string()
                                                        .chars()
                                                        .filter(|c| !c.is_whitespace())
                                                        .collect::<String>()
                                                        .replace("\\n", "");

                                                    let toks =
                                                        arg.split("::").map(|tok| tok.trim()).collect::<Vec<_>>();

                                                    match toks[0] {
                                                        "interface" => Some(HazardFlowAttr::interface_magic(toks[1])),
                                                        "int" => Some(HazardFlowAttr::int_magic(toks[1])),
                                                        "array" => Some(HazardFlowAttr::array_magic(toks[1])),
                                                        "adt" => Some(HazardFlowAttr::adt_magic(toks[1])),
                                                        "ffi" => Some(HazardFlowAttr::ffi(toks[1])),
                                                        "system" => Some(HazardFlowAttr::system(toks[1])),
                                                        "module" => Some(HazardFlowAttr::module_magic(toks[1])),
                                                        "x" => Some(HazardFlowAttr::ExprMagic(ExprMagic::X)),
                                                        _ => panic!("{:?}", toks),
                                                    }
                                                }
                                                _ => todo!("{:?} {:?}", t, s),
                                            }
                                        }
                                        rustc_ast::tokenstream::TokenTree::Delimited(..) => {
                                            todo!()
                                        }
                                    }
                                }
                                rustc_ast::AttrArgs::Eq(..) => todo!(),
                                rustc_ast::AttrArgs::Empty => todo!(),
                            },
                            _ => panic!("Invalid Attribute"),
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    match attrs.len() {
        0 => None,
        1 => Some(attrs[0].clone()),
        _ => panic!(),
    }
}
