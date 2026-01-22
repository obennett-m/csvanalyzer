use crate::types::{CsvErrorType, DataType};
use serde::Serialize;

/// Success response JSON structure
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SuccessResponse {
    pub skip_header: bool,
    pub locale: String,
    pub charset: String,
    pub field_separator: String,
    pub text_delimiter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_time_format: Option<String>,
    pub header_names: Vec<String>,
    pub field_names: Vec<String>,
    pub data_types: Vec<DataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<Vec<String>>>,
}

impl SuccessResponse {
    pub fn new(locale: String, charset: String) -> Self {
        SuccessResponse {
            skip_header: true,
            locale,
            charset,
            field_separator: String::new(),
            text_delimiter: String::new(),
            date_time_format: None,
            header_names: Vec::new(),
            field_names: Vec::new(),
            data_types: Vec::new(),
            data: None,
        }
    }

    /// Set field separator as hex string
    pub fn set_field_separator(&mut self, sep: char) {
        if sep != '\0' {
            self.field_separator = format!("{:02X}", sep as u8);
        }
    }

    /// Set text delimiter as hex string
    pub fn set_text_delimiter(&mut self, delim: char) {
        if delim != '\0' {
            self.text_delimiter = format!("{:02X}", delim as u8);
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Error response JSON structure
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorResponse {
    pub error: u8,
    pub error_msg_user: String,
    pub error_msg_internal: String,
    pub error_row: usize,
    pub error_column: usize,
    pub error_field: String,
    pub error_data_type: u8,
    pub error_column_count: usize,
    pub skip_header: bool,
    pub locale: String,
    pub charset: String,
    pub field_separator: String,
    pub text_delimiter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_types: Option<Vec<DataType>>,
}

impl ErrorResponse {
    pub fn new(error_type: CsvErrorType, locale: String, charset: String) -> Self {
        ErrorResponse {
            error: error_type as u8,
            error_msg_user: error_type.message().to_string(),
            error_msg_internal: String::new(),
            error_row: 0,
            error_column: 0,
            error_field: String::new(),
            error_data_type: 0,
            error_column_count: 0,
            skip_header: false,
            locale,
            charset,
            field_separator: String::new(),
            text_delimiter: String::new(),
            header_names: None,
            field_names: None,
            data_types: None,
        }
    }

    /// Set internal error message
    pub fn with_internal_message(mut self, msg: String) -> Self {
        self.error_msg_internal = msg;
        self
    }

    /// Set error location
    pub fn with_location(mut self, row: usize, col: usize) -> Self {
        self.error_row = row;
        self.error_column = col;
        self
    }

    /// Set error field
    pub fn with_field(mut self, field: String) -> Self {
        self.error_field = field;
        self
    }

    /// Set error data type
    pub fn with_data_type(mut self, dt: DataType) -> Self {
        self.error_data_type = dt as u8;
        self
    }

    /// Set error column count
    pub fn with_column_count(mut self, count: usize) -> Self {
        self.error_column_count = count;
        self
    }

    /// Set field separator as hex string
    pub fn with_field_separator(mut self, sep: char) -> Self {
        if sep != '\0' {
            self.field_separator = format!("{:02X}", sep as u8);
        }
        self
    }

    /// Set text delimiter as hex string
    pub fn with_text_delimiter(mut self, delim: char) -> Self {
        if delim != '\0' {
            self.text_delimiter = format!("{:02X}", delim as u8);
        }
        self
    }

    /// Set header names
    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.header_names = Some(headers);
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response_json() {
        let mut response = SuccessResponse::new("en_US".to_string(), "utf8".to_string());
        response.set_field_separator(',');
        response.set_text_delimiter('"');
        response.header_names = vec!["email".to_string(), "name".to_string()];
        response.field_names = vec!["email".to_string(), "firstname".to_string()];
        response.data_types = vec![DataType::String, DataType::String];

        let json = response.to_json();
        assert!(json.contains("\"FieldSeparator\":\"2C\""));
        assert!(json.contains("\"TextDelimiter\":\"22\""));
        assert!(json.contains("\"HeaderNames\""));
    }

    #[test]
    fn test_error_response_json() {
        let response = ErrorResponse::new(
            CsvErrorType::EmailNotFound,
            "en_US".to_string(),
            "utf8".to_string(),
        );

        let json = response.to_json();
        assert!(json.contains("\"Error\":9"));
        assert!(json.contains("\"ErrorMsgUser\":\"Email column not found\""));
    }

    #[test]
    fn test_hex_encoding() {
        let mut response = SuccessResponse::new("en_US".to_string(), "utf8".to_string());
        response.set_field_separator(';');
        assert_eq!(response.field_separator, "3B");

        response.set_field_separator('\t');
        assert_eq!(response.field_separator, "09");
    }
}
