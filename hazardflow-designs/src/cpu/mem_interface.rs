//! Memory.

/// Memory operation function (load or store)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemOpFcn {
    /// Load
    Load,

    /// Store
    Store,
}

/// Memory operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemOpTyp {
    /// Byte
    B = 1,

    /// Half
    H = 2,

    /// Word
    W = 3,

    /// Double
    D = 4,

    /// Byte unsigned
    BU = 5,

    /// Half unsigned
    HU = 6,

    /// word unsigned
    WU = 7,
}

/// Memory request.
#[derive(Debug, Clone, Copy)]
pub struct MemReq {
    /// address
    pub addr: u32,

    /// data
    pub data: u32,

    /// Memory Function Code
    pub fcn: MemOpFcn,

    /// Memory Type
    pub typ: MemOpTyp,
}

impl MemReq {
    /// Creates a new load request.
    #[inline]
    pub fn load(addr: u32, typ: MemOpTyp) -> Self {
        Self { addr, data: 0, fcn: MemOpFcn::Load, typ }
    }

    /// Creates a new store request.
    #[inline]
    pub fn store(addr: u32, data: u32, typ: MemOpTyp) -> Self {
        Self { addr, data, fcn: MemOpFcn::Store, typ }
    }
}

/// Memory Response.
#[derive(Debug, Clone, Copy)]
pub struct MemRespWithAddr {
    /// data
    pub data: u32,

    /// address
    pub addr: u32,
}

/// Memory Response.
#[derive(Debug, Clone, Copy)]
pub struct MemResp {
    /// data
    pub data: u32,
}
