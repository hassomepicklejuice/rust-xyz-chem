use thiserror::Error;

/// Result type with dynamic error
pub type Result<T> = std::result::Result<T, FileParseError>;

#[derive(Debug, Error)]
pub enum FileParseError {
    #[error("Expected atom data, but found empty line at line {0}")]
    EmptyLine(usize),

    #[error("Expected atom symbol, but found none at line {0}")]
    NoAtomSymbol(usize),

    #[error("Expected position data, but found none at line {0}")]
    NoPositionData(usize),

    #[error("Could not parse data as atom count (usize) at line {0}")]
    InvalidAtomCount(usize),

    #[error("Could not parse data as atom position (f64) at line {0}")]
    InvalidPositionData(usize),

    #[error("{source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Expected empty line, found data at line {0}")]
    UnexpectedData(usize),
}
