//! Generates target code from ShakeFlow module.

use std::collections::VecDeque;
use std::ops::*;

use itertools::izip;

use super::*;
use crate::some_or;
use crate::utils::*;

/// Composite of expressions.
#[derive(Debug, Clone)]
pub enum CompositeExpr<V: Clone> {
    /// Struct of expressions.
    Struct(Vec<CompositeExpr<V>>),

    /// Expression.
    Bits(V),
}

impl<V: Clone + std::fmt::Debug> CompositeExpr<V> {
    /// Converts into expression.
    pub fn into_expr(self) -> V {
        match self {
            Self::Struct(_) => panic!("Cannot convert struct of expressions into expression."),
            Self::Bits(expr) => expr,
        }
    }

    /// Iterator for `CompositeExpr`.
    pub fn iter(&self) -> CompositeExprIterator<V> {
        self.into_iter()
    }

    /// Converts primitive expressions in the tree.
    pub fn map<W: Clone, F: FnMut(V) -> W>(self, mut f: F) -> CompositeExpr<W> {
        CompositeExprMap { inner: self, f: &mut f }.collect()
    }

    /// Zips with other composite expr. Structures of the two compositions should be same.
    pub fn zip<W: Clone + std::fmt::Debug>(self, other: CompositeExpr<W>) -> CompositeExpr<(V, W)> {
        match (self, other) {
            (CompositeExpr::Struct(exprs_self), CompositeExpr::Struct(exprs_other)) => CompositeExpr::Struct(
                izip!(exprs_self.into_iter(), exprs_other.into_iter())
                    .map(|(expr_lhs, expr_rhs)| expr_lhs.zip(expr_rhs))
                    .collect(),
            ),
            (CompositeExpr::Bits(expr_self), CompositeExpr::Bits(expr_other)) => {
                CompositeExpr::Bits((expr_self, expr_other))
            }
            (CompositeExpr::Struct(exprs_self), CompositeExpr::Bits(expr_other)) => panic!("zip: two compositions CompositeExpr::Struct(\n{exprs_self:#?})\nand CompositeExpr::Bits(\n{expr_other:#?})\nhave different structure"),
            (CompositeExpr::Bits(exprs_self), CompositeExpr::Struct(expr_other)) => panic!("zip: two compositions CompositeExpr::Bits(\n{exprs_self:#?})\nand CompositeExpr::Struct(\n{expr_other:#?})\nhave different structure"),
        }
    }
}

#[derive(Debug)]
struct CompositeExprMap<'a, V: Clone, F> {
    inner: CompositeExpr<V>,
    f: &'a mut F,
}

impl<'a, V: Clone, W: Clone, F> CompositeExprMap<'a, V, F>
where F: FnMut(V) -> W
{
    fn collect(self) -> CompositeExpr<W> {
        match self.inner {
            CompositeExpr::Struct(inner) => CompositeExpr::Struct(
                inner.into_iter().map(|expr| CompositeExprMap { inner: expr, f: self.f }.collect()).collect(),
            ),
            CompositeExpr::Bits(expr) => CompositeExpr::Bits((self.f)(expr)),
        }
    }
}

/// Iterator for `CompositeExpr`.
#[derive(Debug)]
pub struct CompositeExprIterator<V> {
    inner: VecDeque<V>,
}

impl<V: Clone> Iterator for CompositeExprIterator<V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_front()
    }
}

impl<V: Clone> IntoIterator for &CompositeExpr<V> {
    type IntoIter = CompositeExprIterator<V>;
    type Item = V;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter_vec = vec![];

        match self {
            CompositeExpr::Struct(inner) => {
                for expr in inner {
                    iter_vec.extend(expr.into_iter().inner)
                }
            }
            CompositeExpr::Bits(expr) => iter_vec.push(expr.clone()),
        }

        Self::IntoIter { inner: iter_vec.into() }
    }
}

impl CompositeExpr<LogicValues> {
    /// Repeats each field in the expressions by n times.
    pub fn repeat(&self, n: usize) -> Self {
        match self {
            CompositeExpr::Struct(inner) => CompositeExpr::Struct(inner.iter().map(|expr| expr.repeat(n)).collect()),
            CompositeExpr::Bits(expr) => CompositeExpr::Bits(LogicValues(expr.0.repeat(n))),
        }
    }
}

impl From<PortDecls> for CompositeExpr<(Option<String>, Shape)> {
    // TODO: Shouldn't we return signedness as well?
    fn from(typ: PortDecls) -> Self {
        match typ {
            PortDecls::Struct(inner) => CompositeExpr::Struct(
                inner
                    .into_iter()
                    .map(|(prefix, typ)| {
                        CompositeExpr::from(typ).map(|(name, shape)| (join_options("_", [prefix.clone(), name]), shape))
                    })
                    .collect(),
            ),
            PortDecls::Bits(shape) => CompositeExpr::Bits((None, shape)),
        }
    }
}

impl CompositeExpr<(String, Shape)> {
    /// Constructs from value type.
    pub fn from_typ(typ: PortDecls, prefix: String) -> Self {
        CompositeExpr::from(typ).map(|(name, shape)| (join_options("_", [Some(prefix.clone()), name]).unwrap(), shape))
    }
}

/// Context.
#[derive(Debug, Default)]
pub struct Context {
    /// Scopes in the context
    scopes: Vec<Scope>,

    /// Genvar index
    genvar_id: usize,

    /// Fsm Cache
    /// XXX: This is a bad design
    pub fsm_cache: FsmCache,

    /// Display tasks
    /// XXX: This is a bad design
    pub displays: Vec<SystemTask>,
}

impl Context {
    /// Creates new context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enters scope with given scope name.
    pub fn enter_scope(&mut self, scope_name: String) {
        self.scopes.push(Scope::new(scope_name));
    }

    /// Leaves scope.
    pub fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    /// Returns prefix of the inner scope.
    pub fn get_prefix(&self) -> Option<String> {
        if self.scopes.is_empty() {
            None
        } else {
            Some(self.scopes.iter().map(|scope| scope.prefix.clone()).collect::<Vec<_>>().join("_"))
        }
    }

    /// Allocates integer.
    pub fn alloc_int_id(&mut self) -> String {
        let count = self.scopes.len();
        assert!(count > 0, "There is no scope in context");
        let int_id = self.scopes[count - 1].int_id;
        self.scopes[count - 1].int_id += 1;
        format!("i{int_id}")
    }

    /// Allocates integer.
    pub fn alloc_int_id_with_prefix(&mut self) -> String {
        join_options("_", [self.get_prefix(), Some(self.alloc_int_id())]).unwrap()
    }

    /// Allocates genvar.
    pub fn alloc_genvar_id(&mut self) -> String {
        let genvar_id = self.genvar_id;
        self.genvar_id += 1;
        format!("g{genvar_id}")
    }

    /// Allocates net or reg.
    pub fn alloc_temp_id(&mut self) -> String {
        let count = self.scopes.len();
        assert!(count > 0, "There is no scope in context");
        let temp_id = self.scopes[count - 1].temp_id;
        self.scopes[count - 1].temp_id += 1;
        format!("t{temp_id}")
    }

    /// Allocates net or reg.
    pub fn alloc_temp_id_with_prefix(&mut self) -> String {
        join_options("_", [self.get_prefix(), Some(self.alloc_temp_id())]).unwrap()
    }

    /// Refreshes fsm cache.
    ///
    /// XXX: This is a bad design
    pub fn clear_fsm_ctx(&mut self) {
        self.fsm_cache.clear();
        self.displays.clear();
    }
}

/// Scope.
#[derive(Debug, Clone)]
pub struct Scope {
    /// Prefix of the scope
    prefix: String,

    /// Integer index
    int_id: usize,

    /// Net, Reg index
    temp_id: usize,
}

impl Scope {
    /// Creates new scope.
    pub fn new(prefix: String) -> Self {
        Self { prefix, int_id: 0, temp_id: 0 }
    }
}

/// Represents port in target language.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Port {
    /// Channel type.
    channel_typ: ChannelTyp,

    /// Array size.
    size: usize,
}

impl Port {
    fn new(channel_typ: ChannelTyp, size: usize) -> Self {
        Port { channel_typ, size }
    }

    fn multiple(self, count: usize) -> Self {
        Port { channel_typ: self.channel_typ, size: self.size * count }
    }
}

/// Direction of port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    /// Input
    Input,

    /// Output
    Output,
}

impl ToString for Direction {
    fn to_string(&self) -> String {
        match self {
            Direction::Input => "input".to_string(),
            Direction::Output => "output".to_string(),
        }
    }
}

/// Accessor to the element in the interface.
#[derive(Default, Debug, Clone)]
struct Accessor {
    /// Prefix.
    prefix: Option<String>,

    /// Separator.
    sep: Option<String>,

    /// Index and total number of elements.
    index: Option<(usize, usize)>,

    /// Trace of array sizes.
    arr_trace: Vec<Option<usize>>,
}

/// Logic value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicValue {
    /// Logic '0' or false condition
    False,
    /// Logic '1' or true condition
    True,
    /// Don't care or unknown value
    X,
    /// High impedance state (used for tri-state buffer)
    Z,
}

impl From<bool> for LogicValue {
    fn from(value: bool) -> Self {
        match value {
            true => Self::True,
            false => Self::False,
        }
    }
}

impl ToString for LogicValue {
    fn to_string(&self) -> String {
        match self {
            LogicValue::False => "0",
            LogicValue::True => "1",
            LogicValue::X => "x",
            LogicValue::Z => "z",
        }
        .to_string()
    }
}

/// Logic values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicValues(Vec<LogicValue>);

impl ToString for LogicValues {
    fn to_string(&self) -> String {
        self.0.iter().map(|b| b.to_string()).collect::<String>()
    }
}

impl Deref for LogicValues {
    type Target = [LogicValue];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl LogicValues {
    /// Creates new logic values.
    pub fn new(inner: Vec<LogicValue>) -> Self {
        Self(inner)
    }

    /// Inner logic values.
    pub fn into_inner(self) -> Vec<LogicValue> {
        self.0
    }
}

/// Returns a set of ports to represent given interface type.
fn gen_ports(interface_typ: &InterfaceTyp) -> Vec<(Port, Accessor)> {
    match interface_typ {
        InterfaceTyp::Unit => Vec::new(),
        InterfaceTyp::Channel(channel_typ) => {
            vec![(Port::new(channel_typ.clone(), 1), Accessor::default())]
        }
        InterfaceTyp::Array(interface_typ, count) => {
            gen_ports(interface_typ).into_iter().map(|(port, accessor)| (port.multiple(*count), accessor)).collect()
        }
        InterfaceTyp::Struct(inner) => inner
            .into_iter()
            .flat_map(|(name, (sep, interface_typ))| {
                gen_ports(interface_typ).into_iter().map(|(port, mut accessor)| {
                    match accessor.prefix {
                        Some(prefix) => {
                            let sep = sep.clone().unwrap_or_else(|| "_".to_string());
                            accessor.prefix = join_options(&sep, [Some(name.clone()), Some(prefix)]);
                        }
                        None => {
                            accessor.prefix = Some(name.clone());
                            accessor.sep = sep.clone();
                        }
                    }
                    (port, accessor)
                })
            })
            .collect(),
    }
}

/// Returns connections in the module instantiation.
///
/// # Returns
///
/// - `Direction`: Direction of the port
/// - `String`: Name of the port
/// - `String`: Name of the expression
pub(super) fn gen_connections<M: PrimitiveModule>(
    module: &M,
    ctx: &mut Context,
) -> VirgenResult<Vec<(Direction, String, String)>> {
    let mut connections = Vec::new();

    connections.push((Direction::Input, "clk".to_string(), "clk".to_string()));
    connections.push((Direction::Input, "rst".to_string(), "rst".to_string()));

    for (port, accessor) in gen_ports(&module.input_interface_typ()) {
        let (path_prefix, path_sep) = (accessor.prefix, accessor.sep);
        let path_sep = path_sep.unwrap_or_else(|| "_".to_string());
        let lvalue_prefix = join_options("_", [Some("in".to_string()), path_prefix.clone()]);
        let rvalue_prefix = join_options("_", [ctx.get_prefix(), Some("in".to_string()), path_prefix]);

        for (name, _) in port.channel_typ.fwd.iter() {
            connections.push((
                Direction::Input,
                join_options(&path_sep, [lvalue_prefix.clone(), Some("payload".to_string()), name.clone()]).unwrap(),
                join_options(&path_sep, [rvalue_prefix.clone(), Some("payload".to_string()), name]).unwrap(),
            ));
        }

        for (name, _) in port.channel_typ.bwd.iter() {
            connections.push((
                Direction::Output,
                join_options(&path_sep, [lvalue_prefix.clone(), Some("resolver".to_string()), name.clone()]).unwrap(),
                join_options(&path_sep, [rvalue_prefix.clone(), Some("resolver".to_string()), name.clone()]).unwrap(),
            ));
        }
    }

    for (port, accessor) in gen_ports(&module.output_interface_typ()) {
        let (path_prefix, path_sep) = (accessor.prefix, accessor.sep);
        let path_sep = path_sep.unwrap_or_else(|| "_".to_string());
        let lvalue_prefix = join_options("_", [Some("out".to_string()), path_prefix.clone()]);
        let rvalue_prefix = join_options("_", [ctx.get_prefix(), Some("out".to_string()), path_prefix]);

        for (name, _) in port.channel_typ.fwd.iter() {
            connections.push((
                Direction::Output,
                join_options(&path_sep, [lvalue_prefix.clone(), Some("payload".to_string()), name.clone()]).unwrap(),
                join_options(&path_sep, [rvalue_prefix.clone(), Some("payload".to_string()), name]).unwrap(),
            ));
        }

        for (name, _) in port.channel_typ.bwd.iter() {
            connections.push((
                Direction::Input,
                join_options(&path_sep, [lvalue_prefix.clone(), Some("resolver".to_string()), name.clone()]).unwrap(),
                join_options(&path_sep, [rvalue_prefix.clone(), Some("resolver".to_string()), name]).unwrap(),
            ));
        }
    }

    Ok(connections)
}

/// Returns port declarations in the module.
///
/// # Returns
///
/// - `Direction`: Direction of the port (input or output)
/// - `usize`: Bitwidth of the port
/// - `String`: Name of the port
#[allow(clippy::needless_lifetimes)]
pub(super) fn gen_port_decls<'tcx>(module: &Virgen<'tcx>) -> VirgenResult<Vec<(Direction, usize, String)>> {
    let mut port_decls = vec![(Direction::Input, 1, "clk".to_string()), (Direction::Input, 1, "rst".to_string())];

    // Port declarations for input interface
    for (port, accessor) in gen_ports(&module.input_interface_typ()) {
        let (path_prefix, path_sep) = (accessor.prefix, accessor.sep);
        let path_sep = path_sep.unwrap_or_else(|| "_".to_string());
        let input_prefix = join_options("_", [Some("in".to_string()), path_prefix]);

        for (name, shape) in port.channel_typ.fwd.iter() {
            assert_eq!(shape.dim(), 1, "Port of module should be 1-dimensional.");
            port_decls.push((
                Direction::Input,
                shape.width() * port.size,
                join_options(&path_sep, [input_prefix.clone(), Some("payload".to_string()), name]).unwrap(),
            ));
        }

        for (name, shape) in port.channel_typ.bwd.iter() {
            assert_eq!(shape.dim(), 1, "Port of module should be 1-dimensional.");
            port_decls.push((
                Direction::Output,
                shape.width() * port.size,
                join_options(&path_sep, [input_prefix.clone(), Some("resolver".to_string()), name]).unwrap(),
            ));
        }
    }

    // Port declarations for output interface
    for (port, accessor) in gen_ports(&module.output_interface_typ()) {
        let (path_prefix, path_sep) = (accessor.prefix, accessor.sep);
        let path_sep = path_sep.unwrap_or_else(|| "_".to_string());
        let output_prefix = join_options("_", [Some("out".to_string()), path_prefix]);

        for (name, shape) in port.channel_typ.fwd.iter() {
            assert_eq!(shape.dim(), 1, "Port of module should be 1-dimensional.");
            port_decls.push((
                Direction::Output,
                shape.width() * port.size,
                join_options(&path_sep, [output_prefix.clone(), Some("payload".to_string()), name]).unwrap(),
            ));
        }

        for (name, shape) in port.channel_typ.bwd.iter() {
            assert_eq!(shape.dim(), 1, "Port of module should be 1-dimensional.");
            port_decls.push((
                Direction::Input,
                shape.width() * port.size,
                join_options(&path_sep, [output_prefix.clone(), Some("resolver".to_string()), name]).unwrap(),
            ));
        }
    }

    Ok(port_decls)
}

/// Returns input/output wires for submodules in the module.
///
/// # Returns
///
/// - `String`: Name of the wire
/// - `Shape`: Shape of the wire
#[allow(clippy::needless_lifetimes)]
pub(super) fn gen_submodule_wires<'tcx>(
    module: &Virgen<'tcx>,
    ctx: &mut Context,
) -> VirgenResult<Vec<(String, String, Shape)>> {
    // Add input/output wires for submodules
    let mut submodule_wires = vec![];

    for (index, (submodule, _)) in module.submodules.iter().enumerate() {
        let comp_name = submodule.get_module_name();
        // Add input wires
        for (port, accessor) in gen_ports(&submodule.input_interface_typ()) {
            let (path_prefix, path_sep) = (accessor.prefix, accessor.sep);
            let path_sep = path_sep.unwrap_or_else(|| "_".to_string());
            let input_prefix =
                join_options("_", [ctx.get_prefix(), Some(format!("{comp_name}_{index}_in")), path_prefix]);

            for (name, shape) in port.channel_typ.fwd.iter() {
                submodule_wires.push((
                    format!("{comp_name}_{index} ingress payload"),
                    join_options(&path_sep, [input_prefix.clone(), Some("payload".to_string()), name]).unwrap(),
                    shape.multiple(port.size),
                ));
            }

            for (name, shape) in port.channel_typ.bwd.iter() {
                submodule_wires.push((
                    format!("{comp_name}_{index} ingress resolver"),
                    join_options(&path_sep, [input_prefix.clone(), Some("resolver".to_string()), name]).unwrap(),
                    shape.multiple(port.size),
                ));
            }
        }

        // Add output wires
        for (port, accessor) in gen_ports(&submodule.output_interface_typ()) {
            let (path_prefix, path_sep) = (accessor.prefix, accessor.sep);
            let path_sep = path_sep.unwrap_or_else(|| "_".to_string());
            let output_prefix =
                join_options("_", [ctx.get_prefix(), Some(format!("{comp_name}_{index}_out")), path_prefix]);

            for (name, shape) in port.channel_typ.fwd.iter() {
                submodule_wires.push((
                    format!("{comp_name}_{index} egress payload ports"),
                    join_options(&path_sep, [output_prefix.clone(), Some("payload".to_string()), name]).unwrap(),
                    shape.multiple(port.size),
                ));
            }

            for (name, shape) in port.channel_typ.bwd.iter() {
                submodule_wires.push((
                    format!("{comp_name}_{index} egress resolver ports"),
                    join_options(&path_sep, [output_prefix.clone(), Some("resolver".to_string()), name]).unwrap(),
                    shape.multiple(port.size),
                ));
            }
        }
    }

    Ok(submodule_wires)
}

/// Returns accessor to channel in interface.
fn gen_channel_accessor(interface_typ: &InterfaceTyp, mut path: EndpointPath) -> Accessor {
    if path.is_empty() {
        assert!(matches!(interface_typ, InterfaceTyp::Channel(_)));
        return Accessor::default();
    }

    let front = path.pop_front().unwrap();
    match (&front, interface_typ) {
        (EndpointNode::Index(i), InterfaceTyp::Array(interface_typ_elt, count)) => {
            let mut accessor = gen_channel_accessor(interface_typ_elt, path);
            accessor.index = match accessor.index {
                Some((index, total)) => Some((total * i + index, total * count)),
                None => Some((*i, *count)),
            };
            accessor.arr_trace.push(Some(*count));
            accessor
        }
        (EndpointNode::Field(name, _), InterfaceTyp::Struct(inner)) => {
            let (sep, interface_typ_field) = inner.get(name).unwrap();
            let mut accessor = gen_channel_accessor(interface_typ_field, path);
            match accessor.prefix {
                Some(prefix) => {
                    accessor.prefix = join_options(&sep.clone().unwrap_or_else(|| "_".to_string()), [
                        Some(name.clone()),
                        Some(prefix),
                    ]);
                }
                None => {
                    accessor.prefix = Some(name.clone());
                    accessor.sep = sep.clone();
                }
            }
            accessor.arr_trace.push(None);
            accessor
        }
        _ => {
            panic!("unmatched endpoint node and interface type: {front:#?} and {interface_typ:#?}")
        }
    }
}

/// Generates bitarrays representing Expr. Panics if it cannot be converted into bitarrays.
///
/// Returned string contains "0", "1" and "x".
pub(super) fn gen_expr_literal(expr: &Expr) -> CompositeExpr<LogicValues> {
    match expr {
        Expr::X { typ, span } => match typ {
            PortDecls::Bits(shape) => CompositeExpr::Bits(LogicValues(vec![LogicValue::X; shape.width()])),
            PortDecls::Struct(inner) => CompositeExpr::Struct(
                inner.iter().map(|(_, typ)| gen_expr_literal(&Expr::X { typ: typ.clone(), span: *span })).collect(),
            ),
        },
        Expr::Constant { bits, typ, span } => match typ {
            PortDecls::Bits(_) => CompositeExpr::Bits(LogicValues(
                bits.iter().rev().map(|x| if *x { LogicValue::True } else { LogicValue::False }).collect(),
            )),
            PortDecls::Struct(inner) => {
                let mut member_exprs = Vec::new();
                let mut offset = 0;

                for (_, typ) in inner {
                    let width = typ.width();
                    member_exprs.push(gen_expr_literal(&Expr::Constant {
                        bits: bits[offset..(offset + width)].to_vec(),
                        typ: typ.clone(),
                        span: *span,
                    }));
                    offset += width;
                }

                CompositeExpr::Struct(member_exprs)
            }
        },
        Expr::Struct { inner, .. } => {
            CompositeExpr::Struct(inner.iter().map(|(_, s)| gen_expr_literal(&s.into_expr())).collect())
        }
        Expr::Repeat { inner, count, .. } => gen_expr_literal(&inner.into_expr()).repeat(*count),
        Expr::Member { inner, index, .. } => {
            let inner = gen_expr_literal(&inner.into_expr());
            match inner {
                CompositeExpr::Struct(inner) => inner[*index].clone(),
                _ => todo!(),
            }
        }
        _ => todo!("not yet implemented: {:?}", expr),
    }
}

/// Returns wirings in the module.
///
/// # Returns
///
/// - `String`: Name of lvalue
/// - `Option<(usize, usize)>`: Index/element size of lvalue
/// - `String`: Name of rvalue
/// - `Option<(usize, usize)>`: Index/element size of rvalue
#[allow(clippy::needless_lifetimes)]
#[allow(clippy::type_complexity)]
pub(super) fn gen_wiring<'tcx>(
    module: &Virgen<'tcx>,
    prefix: Option<String>,
) -> VirgenResult<Vec<(String, Option<(usize, usize)>, String, Option<(usize, usize)>)>> {
    let mut conts = Vec::new();

    // Connections from input interface of the module and output interfaces of submodules in the module.
    let mut input_connections = Vec::new();
    let mut comp_connections = vec![Vec::new(); module.submodules.len()];

    for (submodule_index, (submodule, submodule_inp_interface)) in module.submodules.iter().enumerate() {
        for (submodule_inp_subinterface, path) in submodule_inp_interface.clone().into_primitives() {
            let channel = some_or!(submodule_inp_subinterface.clone().get_channel(), unreachable!());

            let mut comp_accessor = gen_channel_accessor(&submodule_inp_interface.typ(), path);
            comp_accessor.prefix = join_options("_", [
                Some(format!("{}_{}", submodule.get_module_name(), submodule_index)),
                Some("in".to_string()),
                comp_accessor.prefix,
            ]);

            match channel.endpoint() {
                Endpoint::Input { path } => {
                    let mut from_accessor = gen_channel_accessor(&module.input_interface_typ(), path);
                    from_accessor.prefix = join_options("_", [Some("in".to_string()), from_accessor.prefix]);

                    input_connections.push((from_accessor, comp_accessor, channel.typ()));
                }
                Endpoint::Submodule { submodule_index, path } => {
                    let mut from_accessor =
                        gen_channel_accessor(&module.submodules[submodule_index].0.output_interface_typ(), path);
                    from_accessor.prefix = join_options("_", [
                        Some(format!("{}_{}", module.submodules[submodule_index].0.get_module_name(), submodule_index)),
                        Some("out".to_string()),
                        from_accessor.prefix,
                    ]);

                    comp_connections[submodule_index].push((from_accessor, comp_accessor, channel.typ()));
                }
            }
        }
    }

    let module_output_interface = module.output_interface()?;
    for (output_subinterface, path) in module_output_interface.clone().into_primitives() {
        let channel = some_or!(output_subinterface.get_channel(), unreachable!());

        let mut output_accessor = gen_channel_accessor(&module.output_interface_typ(), path);
        output_accessor.prefix = join_options("_", [Some("out".to_string()), output_accessor.prefix]);

        match channel.endpoint() {
            Endpoint::Input { path } => {
                let mut from_accessor = gen_channel_accessor(&module.input_interface_typ(), path);
                from_accessor.prefix = join_options("_", [Some("in".to_string()), from_accessor.prefix]);

                input_connections.push((from_accessor, output_accessor, channel.typ()));
            }
            Endpoint::Submodule { submodule_index, path } => {
                let mut from_accessor =
                    gen_channel_accessor(&module.submodules[submodule_index].0.output_interface_typ(), path);
                from_accessor.prefix = join_options("_", [
                    Some(format!("{}_{}", module.submodules[submodule_index].0.get_module_name(), submodule_index,)),
                    Some("out".to_string()),
                    from_accessor.prefix,
                ]);

                comp_connections[submodule_index].push((from_accessor, output_accessor, channel.typ()));
            }
        }
    }

    for (from_accessor, to_accessor, channel_typ) in
        ::std::iter::empty().chain(input_connections.iter()).chain(comp_connections.concat().iter())
    {
        let lvalue_prefix = join_options("_", [prefix.clone(), to_accessor.prefix.clone()]);
        let rvalue_prefix = join_options("_", [prefix.clone(), from_accessor.prefix.clone()]);

        for (name, shape) in channel_typ.fwd.iter() {
            assert_eq!(shape.dim(), 1);
            let to_sep = to_accessor.sep.clone().unwrap_or_else(|| "_".to_string());
            let to_range = to_accessor.index.map(|(index, _)| (index, shape.width()));
            let to_arr_size_product = to_accessor.arr_trace.iter().filter_map(|x| *x).product::<usize>();
            let from_sep = from_accessor.sep.clone().unwrap_or_else(|| "_".to_string());
            let from_range = from_accessor.index.map(|(index, _)| (index, shape.width()));
            let from_arr_size_product = from_accessor.arr_trace.iter().filter_map(|x| *x).product::<usize>();
            conts.push((
                join_options(&to_sep, [lvalue_prefix.clone(), Some("payload".to_string()), name.clone()]).unwrap(),
                if shape.width() == 1 && to_arr_size_product == 1 { None } else { to_range },
                join_options(&from_sep, [rvalue_prefix.clone(), Some("payload".to_string()), name]).unwrap(),
                if shape.width() == 1 && from_arr_size_product == 1 { None } else { from_range },
            ));
        }

        for (name, shape) in channel_typ.bwd.iter() {
            assert_eq!(shape.dim(), 1);
            let from_sep = from_accessor.sep.clone().unwrap_or_else(|| "_".to_string());
            let from_range = from_accessor.index.map(|(index, _)| (index, shape.width()));
            let from_arr_size_product = from_accessor.arr_trace.iter().filter_map(|x| *x).product::<usize>();
            let to_sep = to_accessor.sep.clone().unwrap_or_else(|| "_".to_string());
            let to_range = to_accessor.index.map(|(index, _)| (index, shape.width()));
            let to_arr_size_product = to_accessor.arr_trace.iter().filter_map(|x| *x).product::<usize>();
            conts.push((
                join_options(&from_sep, [rvalue_prefix.clone(), Some("resolver".to_string()), name.clone()]).unwrap(),
                if shape.width() == 1 && from_arr_size_product == 1 { None } else { from_range },
                join_options(&to_sep, [lvalue_prefix.clone(), Some("resolver".to_string()), name]).unwrap(),
                if shape.width() == 1 && to_arr_size_product == 1 { None } else { to_range },
            ));
        }
    }

    Ok(conts)
}

/// Returns declarations for (IP/EB/EP/IB) given `FSM`
///
/// # Return
/// - Shape: Shape of the declatation
/// - String: Name of the wire declatation
/// - String: Name of the reg declatation
#[allow(clippy::type_complexity)]
pub(super) fn gen_fsm_identifiers(
    module: &Fsm<'_>,
    ctx: &Context,
) -> VirgenResult<(
    Vec<(Shape, String, String)>,
    Vec<(Shape, String, String)>,
    Vec<(Shape, String, String)>,
    Vec<(Shape, String, String)>,
)> {
    let mut ip = vec![];
    let mut eb = vec![];
    let mut ep = vec![];
    let mut ib = vec![];

    let ingress_interface = module.input_interface_typ();

    for (port, accessor) in gen_ports(&ingress_interface) {
        let (path_prefix, path_sep) = (accessor.prefix, accessor.sep);
        let path_sep = path_sep.unwrap_or_else(|| "_".to_string());
        let wire_prefix =
            join_options("_", [ctx.get_prefix(), join_options("_", [Some("in".to_string())]), path_prefix.clone()]);
        let ip_prefix = join_options("_", [ctx.get_prefix(), Some("ip".to_string()), path_prefix.clone()]);

        for (name, shape) in port.channel_typ.fwd.iter() {
            assert_eq!(shape.dim(), 1);
            ip.push((
                shape,
                join_options(&path_sep, [wire_prefix.clone(), Some("payload".to_string()), name.clone()]).unwrap(),
                join_options(&path_sep, [ip_prefix.clone(), name]).unwrap(),
            ));
        }

        let ib_prefix = join_options("_", [ctx.get_prefix(), Some("ib".to_string()), path_prefix]);
        for (name, shape) in port.channel_typ.bwd.iter() {
            assert_eq!(shape.dim(), 1);
            ib.push((
                shape,
                join_options(&path_sep, [wire_prefix.clone(), Some("resolver".to_string()), name.clone()]).unwrap(),
                join_options(&path_sep, [ib_prefix.clone(), name]).unwrap(),
            ));
        }
    }

    let egress_interface = module.output_interface_typ();
    for (port, accessor) in gen_ports(&egress_interface) {
        let (path_prefix, path_sep) = (accessor.prefix, accessor.sep);
        let path_sep = path_sep.unwrap_or_else(|| "_".to_string());
        let wire_prefix = join_options("_", [ctx.get_prefix(), Some("out".to_string()), path_prefix.clone()]);

        let ep_prefix = join_options("_", [ctx.get_prefix(), Some("ep".to_string()), path_prefix.clone()]);
        for (name, shape) in port.channel_typ.fwd.iter() {
            assert_eq!(shape.dim(), 1);
            ep.push((
                shape,
                join_options(&path_sep, [wire_prefix.clone(), Some("payload".to_string()), name.clone()]).unwrap(),
                join_options(&path_sep, [ep_prefix.clone(), name]).unwrap(),
            ));
        }
        let eb_prefix = join_options("_", [ctx.get_prefix(), Some("eb".to_string()), path_prefix]);
        for (name, shape) in port.channel_typ.bwd.iter() {
            assert_eq!(shape.dim(), 1);
            eb.push((
                shape,
                join_options(&path_sep, [wire_prefix.clone(), Some("resolver".to_string()), name.clone()]).unwrap(),
                join_options(&path_sep, [eb_prefix.clone(), name]).unwrap(),
            ));
        }
    }

    Ok((ip, eb, ep, ib))
}

pub(super) fn gen_module_split_assigns(
    m: &ModuleSplit<'_>,
    ctx: &mut Context,
) -> VirgenResult<Vec<(Direction, String, String)>> {
    assert!(m.sig.params.len() == 1);

    let input_interface_typ = m.sig.input_interface_typ();
    let output_interface_typ = m.sig.output_interface_typ();

    let in_out = input_interface_typ
        .get_subinterface(EndpointPath::default().append_node(EndpointNode::Field("output".to_string(), None)));
    let out_in = output_interface_typ
        .get_subinterface(EndpointPath::default().append_node(EndpointNode::Field("input".to_string(), None)));

    let in_out_ports = gen_ports(&in_out);
    let out_in_ports = gen_ports(&out_in);

    let mut conts = vec![];
    for ((incoming_port, incoming_accessor), (outgoing_port, outgoing_accessor)) in izip!(in_out_ports, out_in_ports) {
        log::debug!("incoming: {:#?}", incoming_port);
        log::debug!("outgoing: {:#?}", outgoing_port);

        assert_eq!(incoming_port, outgoing_port);

        let (incoming_path_prefix, incoming_path_sep) = (incoming_accessor.prefix, incoming_accessor.sep);
        let incoming_path_sep = incoming_path_sep.unwrap_or_else(|| "_".to_string());
        let incoming_prefix = join_options("_", [
            ctx.get_prefix(),
            Some("in".to_string()),
            Some("output".to_string()),
            incoming_path_prefix.clone(),
        ]);

        let (outgoing_path_prefix, outgoing_path_sep) = (outgoing_accessor.prefix, outgoing_accessor.sep);
        let outgoing_path_sep = outgoing_path_sep.unwrap_or_else(|| "_".to_string());
        let outgoing_prefix = join_options("_", [
            ctx.get_prefix(),
            Some("out".to_string()),
            Some("input".to_string()),
            outgoing_path_prefix.clone(),
        ]);

        for (name, shape) in incoming_port.channel_typ.fwd.iter() {
            assert_eq!(shape.dim(), 1);
            conts.push((
                Direction::Input,
                join_options(&incoming_path_sep, [incoming_prefix.clone(), Some("payload".to_string()), name.clone()])
                    .unwrap(),
                join_options(&outgoing_path_sep, [outgoing_prefix.clone(), Some("payload".to_string()), name.clone()])
                    .unwrap(),
            ));
        }

        for (name, shape) in incoming_port.channel_typ.bwd.iter() {
            assert_eq!(shape.dim(), 1);
            conts.push((
                Direction::Output,
                join_options(&incoming_path_sep, [incoming_prefix.clone(), Some("resolver".to_string()), name.clone()])
                    .unwrap(),
                join_options(&outgoing_path_sep, [outgoing_prefix.clone(), Some("resolver".to_string()), name.clone()])
                    .unwrap(),
            ));
        }
    }

    // wire incoming from ingress to outgoing to egress
    let in_in = input_interface_typ
        .get_subinterface(EndpointPath::default().append_node(EndpointNode::Field("input".to_string(), None)));
    let out_out = output_interface_typ
        .get_subinterface(EndpointPath::default().append_node(EndpointNode::Field("output".to_string(), None)));

    let in_in_ports = gen_ports(&in_in);
    let out_out_ports = gen_ports(&out_out);

    for ((incoming_port, incoming_accessor), (outgoing_port, outgoing_accessor)) in izip!(in_in_ports, out_out_ports) {
        log::debug!("incoming: {:#?}", incoming_port);
        log::debug!("outgoing: {:#?}", outgoing_port);

        assert_eq!(incoming_port, outgoing_port);

        let (incoming_path_prefix, incoming_path_sep) = (incoming_accessor.prefix, incoming_accessor.sep);
        let incoming_path_sep = incoming_path_sep.unwrap_or_else(|| "_".to_string());
        let incoming_prefix = join_options("_", [
            ctx.get_prefix(),
            Some("in".to_string()),
            Some("input".to_string()),
            incoming_path_prefix.clone(),
        ]);

        let (outgoing_path_prefix, outgoing_path_sep) = (outgoing_accessor.prefix, outgoing_accessor.sep);
        let outgoing_path_sep = outgoing_path_sep.unwrap_or_else(|| "_".to_string());
        let outgoing_prefix = join_options("_", [
            ctx.get_prefix(),
            Some("out".to_string()),
            Some("output".to_string()),
            outgoing_path_prefix.clone(),
        ]);

        for (name, shape) in incoming_port.channel_typ.fwd.iter() {
            assert_eq!(shape.dim(), 1);
            conts.push((
                Direction::Input,
                join_options(&incoming_path_sep, [incoming_prefix.clone(), Some("payload".to_string()), name.clone()])
                    .unwrap(),
                join_options(&outgoing_path_sep, [outgoing_prefix.clone(), Some("payload".to_string()), name.clone()])
                    .unwrap(),
            ));
        }

        for (name, shape) in incoming_port.channel_typ.bwd.iter() {
            assert_eq!(shape.dim(), 1);
            conts.push((
                Direction::Output,
                join_options(&incoming_path_sep, [incoming_prefix.clone(), Some("resolver".to_string()), name.clone()])
                    .unwrap(),
                join_options(&outgoing_path_sep, [outgoing_prefix.clone(), Some("resolver".to_string()), name.clone()])
                    .unwrap(),
            ));
        }
    }

    Ok(conts)
}

// Return type: (lvalue_name: String, lvalue_range: Option<(usize, usize)>, rvalue_name: String, rvalue_range: Option<(usize, usize)>)
#[allow(clippy::type_complexity)]
pub(super) fn gen_module_seq_assigns(
    m: &ModuleSeq<'_>,
    ctx: &mut Context,
) -> VirgenResult<Vec<(String, Option<(usize, usize)>, String, Option<(usize, usize)>)>> {
    assert!(m.sig.params.len() == 1, "seq module should have only one parameter");
    let mut connections: Vec<(String, Option<(usize, usize)>, String, Option<(usize, usize)>)> = vec![];

    for (param_idx, param) in m.sig.params.iter().enumerate() {
        // Check the parameters
        assert!(param_idx == 0, "seq module should have only one parameter");
        let ModuleGraphType::ComposedModule(ComposedModuleTy::Array(fn_ptr, seq_len)) = param else {
            panic!("invalid module parameter: {:#?}", param)
        };
        let ModuleGraphType::Module(ref module_sig) = **fn_ptr else {
            panic!("seq module should have function pointer as parameter")
        };
        assert!(
            module_sig.params.len() == 2,
            "input type of seq module's parameter(which is function pointer) should be ([i; N], j)"
        );
        let ModuleGraphType::Interface(ref i_typ) = &module_sig.params[0] else { panic!() };
        let ModuleGraphType::Interface(ref j_typ) = &module_sig.params[1] else { panic!() };
        let ModuleGraphType::Interface(InterfaceTyp::Struct(ref ref_ty)) = *module_sig.ret_ty else {
            panic!("output type of seq module's parameter(which is function pointer) should be ([o; N], j)")
        };
        let Some((_, o_typ)) = ref_ty.get("0") else { panic!() };
        assert!(ref_ty.get("1").is_some_and(|(_, out_j)| out_j == j_typ));

        let input_interface_typ = m.sig.input_interface_typ();
        let output_interface_typ = m.sig.output_interface_typ();

        let in_input = input_interface_typ
            .get_subinterface(EndpointPath::default().append_field("input").append_field(&param_idx.to_string())); // [(o, j); seq_len]
        let in_output = input_interface_typ.get_subinterface(EndpointPath::default().append_field("output")); // ([i; seq_len], j)
        let out_input = output_interface_typ
            .get_subinterface(EndpointPath::default().append_field("input").append_field(&param_idx.to_string())); // [(i, j); seq_len]
        let out_output = output_interface_typ.get_subinterface(EndpointPath::default().append_field("output")); // ([o; seq_len], j)

        // Pseudo-code
        // for idx in seq_len {
        //      // wire `i`
        //      out_input[idx].0 = in_output.0[idx]
        //
        //      // wire `o`
        //      out_output.0[idx] = in_input[idx].0
        //
        //      // wire `j`
        //      if (idx == 0) {
        //          out_input[idx].1 = in_output.1
        //      } else {
        //          out_input[idx].1 = in_input[idx-1].1
        //      }
        // }
        //
        // // wire last `j`
        // out_output.1 = in_input[seq_len-1].1

        for idx in 0..*seq_len {
            // wire `i`
            // assign out_input[idx].0 = in_output.0[idx]
            {
                let tgt_interface_typ = out_input.get_subinterface(
                    EndpointPath::default()
                        .append_index(idx)
                        .append_field("input")
                        .append_field(&param_idx.to_string()),
                ); // out_input[idx].0
                let src_interface_typ = in_output.get_subinterface(
                    EndpointPath::default()
                        .append_field("input")
                        .append_field(&param_idx.to_string())
                        .append_index(idx),
                ); // in_output.0[idx]
                for ((tgt_port, tgt_port_accessor), (src_port, src_port_accessor)) in
                    izip!(gen_ports(&tgt_interface_typ), gen_ports(&src_interface_typ))
                {
                    assert_eq!(tgt_port, src_port);

                    let (tgt_path_prefix, tgt_path_sep) = (tgt_port_accessor.prefix, tgt_port_accessor.sep);
                    let tgt_path_sep = tgt_path_sep.unwrap_or_else(|| "_".to_string());
                    let tgt_prefix = join_options("_", [
                        ctx.get_prefix(),
                        Some("out".to_string()),
                        Some("input".to_string()),
                        Some(param_idx.to_string()),
                        Some("input".to_string()),
                        Some("0".to_string()), // `i`
                        tgt_path_prefix.clone(),
                    ]);
                    let tgt_arr_size_product = tgt_port_accessor.arr_trace.iter().filter_map(|x| *x).product::<usize>();

                    let (src_path_prefix, src_path_sep) = (src_port_accessor.prefix, src_port_accessor.sep);
                    let src_path_sep = src_path_sep.unwrap_or_else(|| "_".to_string());
                    let src_prefix = join_options("_", [
                        ctx.get_prefix(),
                        Some("in".to_string()),
                        Some("output".to_string()),
                        Some("input".to_string()),
                        Some("0".to_string()), // `i`
                        src_path_prefix.clone(),
                    ]);
                    let src_arr_size_product = src_port_accessor.arr_trace.iter().filter_map(|x| *x).product::<usize>();

                    // Wire payload
                    for (name, shape) in tgt_port.channel_typ.fwd.iter() {
                        assert_eq!(shape.dim(), 1);
                        let item_width = if let InterfaceTyp::Array(_, i_len) = i_typ {
                            i_len * shape.width()
                        } else {
                            shape.width()
                        };
                        connections.push((
                            join_options(&tgt_path_sep, [
                                tgt_prefix.clone(),
                                Some("payload".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if item_width == 1 && tgt_arr_size_product == 1 && *seq_len == 1 {
                                None
                            } else {
                                Some((idx, item_width))
                            },
                            join_options(&src_path_sep, [
                                src_prefix.clone(),
                                Some("payload".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if item_width == 1 && src_arr_size_product == 1 && *seq_len == 1 {
                                None
                            } else {
                                Some((idx, item_width))
                            },
                        ));
                    }

                    // Wire resolver
                    for (name, shape) in tgt_port.channel_typ.bwd.iter() {
                        assert_eq!(shape.dim(), 1);
                        let item_width = if let InterfaceTyp::Array(_, i_len) = i_typ {
                            i_len * shape.width()
                        } else {
                            shape.width()
                        };
                        connections.push((
                            join_options(&src_path_sep, [
                                src_prefix.clone(),
                                Some("resolver".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if item_width == 1 && src_arr_size_product == 1 && *seq_len == 1 {
                                None
                            } else {
                                Some((idx, item_width))
                            },
                            join_options(&tgt_path_sep, [
                                tgt_prefix.clone(),
                                Some("resolver".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if item_width == 1 && tgt_arr_size_product == 1 && *seq_len == 1 {
                                None
                            } else {
                                Some((idx, item_width))
                            },
                        ));
                    }
                }
            }

            {
                // wire `o`
                // assign out_output.0[idx] = in_input[idx].0
                let tgt_interface_typ = out_output.get_subinterface(
                    EndpointPath::default().append_field("output").append_field("0").append_index(idx),
                ); // out_output.0[idx]
                let src_interface_typ = in_input.get_subinterface(
                    EndpointPath::default().append_index(idx).append_field("output").append_field("0"),
                ); // in_input[idx].0
                for ((tgt_port, tgt_port_accessor), (src_port, src_port_accessor)) in
                    izip!(gen_ports(&tgt_interface_typ), gen_ports(&src_interface_typ))
                {
                    assert_eq!(tgt_port, src_port);

                    let (tgt_path_prefix, tgt_path_sep) = (tgt_port_accessor.prefix, tgt_port_accessor.sep);
                    let tgt_path_sep = tgt_path_sep.unwrap_or_else(|| "_".to_string());
                    let tgt_prefix = join_options("_", [
                        ctx.get_prefix(),
                        Some("out".to_string()),
                        Some("output".to_string()),
                        Some("output".to_string()),
                        Some("0".to_string()), // `o`
                        tgt_path_prefix.clone(),
                    ]);

                    let (src_path_prefix, src_path_sep) = (src_port_accessor.prefix, src_port_accessor.sep);
                    let src_path_sep = src_path_sep.unwrap_or_else(|| "_".to_string());
                    let src_prefix = join_options("_", [
                        ctx.get_prefix(),
                        Some("in".to_string()),
                        Some("input".to_string()),
                        Some(param_idx.to_string()),
                        Some("output".to_string()),
                        Some("0".to_string()), // `o`
                        src_path_prefix.clone(),
                    ]);

                    // Wire payload
                    for (name, shape) in tgt_port.channel_typ.fwd.iter() {
                        assert_eq!(shape.dim(), 1);
                        let item_width = if let InterfaceTyp::Array(_, o_len) = o_typ {
                            shape.width() * o_len
                        } else {
                            shape.width()
                        };
                        shape.width();
                        connections.push((
                            join_options(&tgt_path_sep, [
                                tgt_prefix.clone(),
                                Some("payload".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if item_width == 1 && *seq_len == 1 { None } else { Some((idx, item_width)) },
                            join_options(&src_path_sep, [
                                src_prefix.clone(),
                                Some("payload".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if item_width == 1 && *seq_len == 1 { None } else { Some((idx, item_width)) },
                        ));
                    }

                    // Wire resolver
                    for (name, shape) in tgt_port.channel_typ.bwd.iter() {
                        assert_eq!(shape.dim(), 1);
                        let item_width = if let InterfaceTyp::Array(_, o_len) = o_typ {
                            shape.width() * o_len
                        } else {
                            shape.width()
                        };
                        connections.push((
                            join_options(&src_path_sep, [
                                src_prefix.clone(),
                                Some("resolver".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if item_width == 1 && *seq_len == 1 { None } else { Some((idx, item_width)) },
                            join_options(&tgt_path_sep, [
                                tgt_prefix.clone(),
                                Some("resolver".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if item_width == 1 && *seq_len == 1 { None } else { Some((idx, item_width)) },
                        ));
                    }
                }
            }

            {
                // wire `j`
                let tgt_interface_typ = out_input.get_subinterface(
                    EndpointPath::default().append_index(idx).append_field("input").append_field("1"),
                ); // out_input[idx].1
                let src_interface_typ = if idx == 0 {
                    // in_output.1
                    in_output.get_subinterface(EndpointPath::default().append_field("input").append_field("1"))
                } else {
                    // in_input[idx-1].1
                    in_input.get_subinterface(
                        EndpointPath::default().append_index(idx - 1).append_field("output").append_field("1"),
                    )
                };
                for ((tgt_port, tgt_port_accessor), (src_port, src_port_accessor)) in
                    izip!(gen_ports(&tgt_interface_typ), gen_ports(&src_interface_typ))
                {
                    assert_eq!(tgt_port, src_port);

                    let (tgt_path_prefix, tgt_path_sep) = (tgt_port_accessor.prefix, tgt_port_accessor.sep);
                    let tgt_path_sep = tgt_path_sep.unwrap_or_else(|| "_".to_string());
                    let tgt_prefix = join_options("_", [
                        ctx.get_prefix(),
                        Some("out".to_string()),
                        Some("input".to_string()),
                        Some(param_idx.to_string()),
                        Some("input".to_string()),
                        Some("1".to_string()),
                        tgt_path_prefix.clone(),
                    ]);

                    let (src_path_prefix, src_path_sep) = (src_port_accessor.prefix, src_port_accessor.sep);
                    let src_path_sep = src_path_sep.unwrap_or_else(|| "_".to_string());
                    let src_prefix = if idx == 0 {
                        join_options("_", [
                            ctx.get_prefix(),
                            Some("in".to_string()),
                            Some("output".to_string()),
                            Some("input".to_string()),
                            Some("1".to_string()),
                            src_path_prefix.clone(),
                        ])
                    } else {
                        join_options("_", [
                            ctx.get_prefix(),
                            Some("in".to_string()),
                            Some("input".to_string()),
                            Some(param_idx.to_string()),
                            Some("output".to_string()),
                            Some("1".to_string()),
                            src_path_prefix.clone(),
                        ])
                    };

                    // Wire payload
                    for (name, shape) in tgt_port.channel_typ.fwd.iter() {
                        assert_eq!(shape.dim(), 1);
                        let item_width = if let InterfaceTyp::Array(..) = j_typ {
                            j_typ.nested_array_flattened_len() * shape.width()
                        } else {
                            shape.width()
                        };
                        connections.push((
                            join_options(&tgt_path_sep, [
                                tgt_prefix.clone(),
                                Some("payload".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if item_width == 1 && *seq_len == 1 { None } else { Some((idx, item_width)) },
                            join_options(&src_path_sep, [
                                src_prefix.clone(),
                                Some("payload".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if idx == 0 {
                                if item_width == 1 {
                                    None
                                } else {
                                    Some((0, item_width))
                                }
                            } else if item_width == 1 && *seq_len == 1 {
                                None
                            } else {
                                Some(((idx - 1), item_width))
                            },
                        ));
                    }

                    // Wire resolver
                    for (name, shape) in tgt_port.channel_typ.bwd.iter() {
                        assert_eq!(shape.dim(), 1);
                        let item_width = if let InterfaceTyp::Array(..) = j_typ {
                            j_typ.nested_array_flattened_len() * shape.width()
                        } else {
                            shape.width()
                        };
                        connections.push((
                            join_options(&src_path_sep, [
                                src_prefix.clone(),
                                Some("resolver".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if (idx == 0) || (item_width == 1 && *seq_len == 1) {
                                None
                            } else {
                                Some((idx - 1, item_width))
                            },
                            join_options(&tgt_path_sep, [
                                tgt_prefix.clone(),
                                Some("resolver".to_string()),
                                name.clone(),
                            ])
                            .unwrap(),
                            if item_width == 1 { None } else { Some((idx, item_width)) },
                        ));
                    }
                }
            }
        }

        {
            // wire last `j`
            // assign out_output.1 = in_input[seq_len-1].1
            let tgt_interface_typ =
                out_output.get_subinterface(EndpointPath::default().append_field("output").append_field("1")); // out_output.1
            let src_interface_typ = in_input.get_subinterface(
                EndpointPath::default().append_index(*seq_len - 1).append_field("output").append_field("1"),
            ); // in_input[seq_len].1
            for ((tgt_port, tgt_port_accessor), (src_port, src_port_accessor)) in
                izip!(gen_ports(&tgt_interface_typ), gen_ports(&src_interface_typ))
            {
                assert_eq!(tgt_port, src_port);

                let (tgt_path_prefix, tgt_path_sep) = (tgt_port_accessor.prefix, tgt_port_accessor.sep);
                let tgt_path_sep = tgt_path_sep.unwrap_or_else(|| "_".to_string());
                let tgt_prefix = join_options("_", [
                    ctx.get_prefix(),
                    Some("out".to_string()),
                    Some("output".to_string()),
                    Some("output".to_string()),
                    Some("1".to_string()), // `j`
                    tgt_path_prefix.clone(),
                ]);

                let (src_path_prefix, src_path_sep) = (src_port_accessor.prefix, src_port_accessor.sep);
                let src_path_sep = src_path_sep.unwrap_or_else(|| "_".to_string());
                let src_prefix = join_options("_", [
                    ctx.get_prefix(),
                    Some("in".to_string()),
                    Some("input".to_string()),
                    Some(param_idx.to_string()),
                    Some("output".to_string()),
                    Some("1".to_string()), // `j`
                    src_path_prefix.clone(),
                ]);

                // Wire payload
                for (name, shape) in tgt_port.channel_typ.fwd.iter() {
                    assert_eq!(shape.dim(), 1);
                    let item_width = if let InterfaceTyp::Array(..) = j_typ {
                        j_typ.nested_array_flattened_len() * shape.width()
                    } else {
                        shape.width()
                    };
                    connections.push((
                        join_options(&tgt_path_sep, [tgt_prefix.clone(), Some("payload".to_string()), name.clone()])
                            .unwrap(),
                        None,
                        join_options(&src_path_sep, [src_prefix.clone(), Some("payload".to_string()), name.clone()])
                            .unwrap(),
                        if item_width == 1 && *seq_len == 1 { None } else { Some((*seq_len - 1, item_width)) },
                    ));
                }

                // Wire resolver
                for (name, shape) in tgt_port.channel_typ.bwd.iter() {
                    assert_eq!(shape.dim(), 1);
                    let item_width = if let InterfaceTyp::Array(..) = j_typ {
                        j_typ.nested_array_flattened_len() * shape.width()
                    } else {
                        shape.width()
                    };
                    connections.push((
                        join_options(&src_path_sep, [src_prefix.clone(), Some("resolver".to_string()), name.clone()])
                            .unwrap(),
                        Some((*seq_len - 1, item_width)),
                        join_options(&tgt_path_sep, [tgt_prefix.clone(), Some("resolver".to_string()), name.clone()])
                            .unwrap(),
                        None,
                    ));
                }
            }
        }
    }

    Ok(connections)
}
