pub mod analyzer;
pub mod config;
pub mod db;
pub mod detection;
pub mod error;
pub mod output;
pub mod types;
pub mod validation;

pub use analyzer::CsvAnalyzer;
pub use config::{Config, DbConfig};
pub use error::{CsvAnalyzerError, Result};
pub use types::{CsvErrorType, DataType};
