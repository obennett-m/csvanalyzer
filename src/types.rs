use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt;

/// Data type codes matching Pascal mjconsts.pas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
#[derive(Default)]
pub enum DataType {
    #[default]
    String = 0,
    Integer = 1,
    Float = 2,
    Boolean = 3,
    DateTime = 4,
}

/// CSV error type codes matching Pascal csvanalyzer.pas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CsvErrorType {
    Process = 0,            // Unhandled exception
    Database = 1,           // Database error
    Sample = 2,             // Could not get sample (empty file)
    Binary = 3,             // File is binary
    VariousFieldsCount = 4, // Too much column count variation
    TooMuchColumns = 5,     // Exceeds max columns
    ColumnLong = 6,         // Column name too long
    ValueLong = 7,          // Field value too long
    DuplicateField = 8,     // Duplicate column name in header
    EmailNotFound = 9,      // No email column detected
}

impl fmt::Display for CsvErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl CsvErrorType {
    pub fn message(&self) -> &'static str {
        match self {
            CsvErrorType::Process => "Unhandled exception",
            CsvErrorType::Database => "Database error",
            CsvErrorType::Sample => "Could not get a sample for analyze. Is file empty?",
            CsvErrorType::Binary => "File is binary file",
            CsvErrorType::VariousFieldsCount => {
                "Can not determine the number of columns. Too much variation in column count"
            }
            CsvErrorType::TooMuchColumns => "Too many columns detected. Maximum %d columns allowed",
            CsvErrorType::ColumnLong => "Column name \"%s\" in column %d is too long",
            CsvErrorType::ValueLong => "Value \"%s\" in row %d, column %d is too long",
            CsvErrorType::DuplicateField => "Duplicate field name \"%s\"",
            CsvErrorType::EmailNotFound => "Email column not found",
        }
    }
}

/// Contact property metadata from database
#[derive(Debug, Clone)]
pub struct ContactProperty {
    pub name: String,
    pub datatype: DataType,
}

/// Date pattern for detection
#[derive(Debug, Clone)]
pub struct DatePattern {
    pub pattern: &'static str,
    pub separator: char,
    pub order: &'static str,
}

/// Time pattern for detection
#[derive(Debug, Clone)]
pub struct TimePattern {
    pub pattern: &'static str,
    pub separator: char,
}

/// Constants
pub mod constants {
    pub const MAX_SCAN_LINES: usize = 1000;
    pub const MAX_RETURN_LINES: usize = 10;
    pub const MAX_COLUMNS: usize = 200;
    pub const MAX_STRING_SIZE: usize = 1000;
    pub const BUFF_SIZE: usize = 10240; // 10KB
    pub const MAX_BYTES: usize = 51200; // 50KB
    pub const FIELD_DELIM_PERCENT: usize = 50;
    pub const TEXT_SEP_PERCENT: usize = 50;
    pub const COLUMN_COUNT_PERCENT: usize = 90;
    pub const MAX_BUCKET: usize = 4;
    pub const CSVA_GUESS_SIZE: usize = 5120; // 5KB threshold for quick charset guess

    /// Candidate field delimiters in priority order
    pub const FIELD_DELIMS: [char; 6] = ['\x0B', ',', ';', '|', ' ', '\t'];

    /// Text separator candidates
    pub const TEXT_SEPS: [char; 2] = ['"', '\''];

    /// Valid characters in email local part
    pub const EMAIL_LOCAL_CHARS: &str = "abcdefghijklmnopqrstuvwxyz0123456789._-+";

    /// Valid characters in email domain
    pub const EMAIL_DOMAIN_CHARS: &str = "abcdefghijklmnopqrstuvwxyz0123456789.-";
}

/// Date patterns matching Pascal mjutils.pas DATE_PATTERNS
pub const DATE_PATTERNS: &[DatePattern] = &[
    DatePattern {
        pattern: "yyyy-mm-dd",
        separator: '-',
        order: "y/m/d",
    },
    DatePattern {
        pattern: "dd-mm-yyyy",
        separator: '-',
        order: "d/m/y",
    },
    DatePattern {
        pattern: "dd/mm/yyyy",
        separator: '/',
        order: "d/m/y",
    },
    DatePattern {
        pattern: "dd.mm.yyyy",
        separator: '.',
        order: "d/m/y",
    },
    DatePattern {
        pattern: "yyyy.dd.mm",
        separator: '.',
        order: "y/m/d",
    },
    DatePattern {
        pattern: "yyyy.mm.dd",
        separator: '.',
        order: "y/m/d",
    },
    DatePattern {
        pattern: "yyyy/mm/dd",
        separator: '/',
        order: "y/m/d",
    },
    DatePattern {
        pattern: "mm/dd/yyyy",
        separator: '/',
        order: "m/d/y",
    },
    DatePattern {
        pattern: "mm.dd.yyyy",
        separator: '.',
        order: "m/d/y",
    },
    DatePattern {
        pattern: "mm-dd-yyyy",
        separator: '-',
        order: "m/d/y",
    },
];

/// Time patterns matching Pascal mjutils.pas TIME_PATTERNS
pub const TIME_PATTERNS: &[TimePattern] = &[
    TimePattern {
        pattern: "hh:nn:ss am/pm",
        separator: ':',
    },
    TimePattern {
        pattern: "hh:nn:ss",
        separator: ':',
    },
    TimePattern {
        pattern: "hh:nn am/pm",
        separator: ':',
    },
    TimePattern {
        pattern: "hh:nn",
        separator: ':',
    },
];
