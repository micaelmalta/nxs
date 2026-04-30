use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum NxsError {
    BadMagic,
    UnknownSigil(char),
    BadEscape(char),
    OutOfBounds,
    DictMismatch,
    CircularLink,
    RecursionLimit,
    MacroUnresolved(String),
    ListTypeMismatch,
    Overflow,
    ParseError(String),
    IoError(String),
}

impl fmt::Display for NxsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NxsError::BadMagic => write!(f, "ERR_BAD_MAGIC"),
            NxsError::UnknownSigil(c) => write!(f, "ERR_UNKNOWN_SIGIL: '{c}'"),
            NxsError::BadEscape(c) => write!(f, "ERR_BAD_ESCAPE: '\\{c}'"),
            NxsError::OutOfBounds => write!(f, "ERR_OUT_OF_BOUNDS"),
            NxsError::DictMismatch => write!(f, "ERR_DICT_MISMATCH"),
            NxsError::CircularLink => write!(f, "ERR_CIRCULAR_LINK"),
            NxsError::RecursionLimit => write!(f, "ERR_RECURSION_LIMIT"),
            NxsError::MacroUnresolved(s) => write!(f, "ERR_MACRO_UNRESOLVED: {s}"),
            NxsError::ListTypeMismatch => write!(f, "ERR_LIST_TYPE_MISMATCH"),
            NxsError::Overflow => write!(f, "ERR_OVERFLOW"),
            NxsError::ParseError(s) => write!(f, "ParseError: {s}"),
            NxsError::IoError(s) => write!(f, "IoError: {s}"),
        }
    }
}

impl std::error::Error for NxsError {}

pub type Result<T> = std::result::Result<T, NxsError>;
