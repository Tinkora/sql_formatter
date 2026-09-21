use thiserror::Error;

/// Stable error type for SQL formatting operations.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    /// A tokenization or syntax-level error occurred during parsing.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// The requested dialect string is not supported.
    #[error("Unsupported dialect: {0}")]
    UnsupportedDialect(String),

    /// The input SQL string is empty or contains only whitespace.
    #[error("Empty input: SQL string is empty or whitespace-only")]
    EmptyInput,

    /// One or more validation issues were found.
    #[error("Validation issues found: {0:?}")]
    ValidationError(Vec<String>),
}

impl CoreError {
    /// Returns a stable machine error code for Web, CLI, and Agent consumers.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ParseError(_) => "PARSE_ERROR",
            Self::UnsupportedDialect(_) => "UNSUPPORTED_DIALECT",
            Self::EmptyInput => "EMPTY_INPUT",
            Self::ValidationError(_) => "VALIDATION_ERROR",
        }
    }
}
