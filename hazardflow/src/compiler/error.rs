//! Virgen Error

use thiserror::Error;

/// Virgen Result
pub type VirgenResult<T> = Result<T, VirgenError>;

/// Virgen Error
#[derive(Debug, Error)]
pub enum VirgenError {
    /// File system error
    #[error("file system error: {err:?}")]
    Fs {
        /// error
        err: std::io::Error,
    },

    /// Collect FSM error
    #[error("Collect FSM error: {msg:?}")]
    CollectFsmError {
        /// Error message
        msg: String,
    },

    /// Port generation error
    #[error("Port generation : {msg:?}")]
    PortGenerationError {
        /// Error message
        msg: String,
    },

    /// TODO: split this Misc to specific error cases
    #[error("Virgen Error Misc: {msg:?}")]
    Misc {
        /// Error message
        msg: String,
    },

    /// Signature error
    #[error("Virgen Error Signature: {msg:?}")]
    InvalidSignature {
        /// Error message
        msg: String,
    },

    /// Analysis error
    #[error("Virgen Error Analysis: {msg:?}")]
    AnalysisError {
        /// Error message
        msg: String,
    },
}

impl VirgenError {
    /// Collect fsm error
    pub(crate) fn collect_fsm_error(msg: String) -> Self {
        VirgenError::CollectFsmError { msg }
    }
}
