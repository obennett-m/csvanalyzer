use crate::types::CsvErrorType;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CsvAnalyzerError {
    #[error("CSV error: {0}")]
    CsvError(CsvErrorType),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),
}

impl CsvAnalyzerError {
    pub fn error_type(&self) -> CsvErrorType {
        match self {
            CsvAnalyzerError::CsvError(t) => *t,
            CsvAnalyzerError::IoError(_) => CsvErrorType::Process,
            CsvAnalyzerError::DatabaseError(_) => CsvErrorType::Database,
            CsvAnalyzerError::ConfigError(_) => CsvErrorType::Process,
            CsvAnalyzerError::EncodingError(_) => CsvErrorType::Process,
        }
    }
}

pub type Result<T> = std::result::Result<T, CsvAnalyzerError>;
