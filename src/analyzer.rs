use crate::config::Config;
use crate::db::{match_property, DbConnection};
use crate::detection::{
    charset::convert_to_utf8, detect_charset, detect_data_type, detect_delimiter,
    detect_email_column, detect_quote_char, has_header,
};
use crate::error::{CsvAnalyzerError, Result};
use crate::output::{ErrorResponse, SuccessResponse};
use crate::types::constants::{
    BUFF_SIZE, MAX_BYTES, MAX_COLUMNS, MAX_RETURN_LINES, MAX_SCAN_LINES,
};
use crate::types::{ContactProperty, CsvErrorType, DataType};
use crate::validation::{
    check_duplicate_fields, is_binary_data, is_valid_string_size, validate_columns_count,
};
use std::fs::File;
use std::io::{BufReader, Read};

/// CSV Analyzer main struct
pub struct CsvAnalyzer {
    config: Config,
    text_sep: char,
    field_delim: char,
    skip_header: bool,
    charset: String,
    current_row: usize,
    current_col: usize,
    current_field: String,
    current_data_type: DataType,
    current_col_count: usize,
}

impl CsvAnalyzer {
    pub fn new(config: Config) -> Self {
        CsvAnalyzer {
            config,
            text_sep: '\0',
            field_delim: '\0',
            skip_header: false,
            charset: "UNKNOWN".to_string(),
            current_row: 0,
            current_col: 0,
            current_field: String::new(),
            current_data_type: DataType::String,
            current_col_count: 0,
        }
    }

    /// Run the analysis and return JSON result
    pub fn analyze(&mut self) -> String {
        match self.analyze_internal() {
            Ok(response) => response.to_json(),
            Err(e) => self.build_error_response(e).to_json(),
        }
    }

    /// Internal analysis implementation
    fn analyze_internal(&mut self) -> Result<SuccessResponse<'_>> {
        // Read sample from file
        let sample = self.read_sample()?;

        // Check for binary file
        if is_binary_data(&sample) {
            return Err(CsvAnalyzerError::CsvError(CsvErrorType::Binary));
        }

        // Detect charset
        self.charset = detect_charset(&sample);

        // Convert to UTF-8
        let text =
            convert_to_utf8(&sample, &self.charset).map_err(CsvAnalyzerError::EncodingError)?;

        // Split into lines
        let lines: Vec<&str> = text.lines().take(MAX_SCAN_LINES + 1).collect();

        if lines.is_empty() {
            return Err(CsvAnalyzerError::CsvError(CsvErrorType::Sample));
        }

        // Detect CSV format
        self.text_sep = detect_quote_char(&lines).unwrap_or('"');
        self.field_delim = detect_delimiter(&lines, Some(self.text_sep)).unwrap_or('\0');
        self.skip_header = has_header(&lines, self.text_sep, self.field_delim);

        // Validate column counts
        let validation = validate_columns_count(&lines, self.field_delim, self.text_sep)?;
        self.current_col_count = validation.columns_count;

        // Check max columns
        if self.current_col_count > MAX_COLUMNS {
            return Err(CsvAnalyzerError::CsvError(CsvErrorType::TooMuchColumns));
        }

        // Parse CSV into rows
        let rows = self.parse_csv(&lines)?;

        // Get headers
        let headers = if self.skip_header && !rows.is_empty() {
            rows[0].clone()
        } else {
            (1..=self.current_col_count)
                .map(|i| format!("Field{}", i))
                .collect()
        };

        // Validate headers
        for (i, header) in headers.iter().enumerate() {
            self.current_col = i + 1;
            if !is_valid_string_size(header) {
                self.current_field = header.clone();
                return Err(CsvAnalyzerError::CsvError(CsvErrorType::ColumnLong));
            }
        }

        // Check for duplicate headers
        check_duplicate_fields(&headers)?;

        // Data rows (skip header if present)
        let data_rows: Vec<Vec<String>> = if self.skip_header && rows.len() > 1 {
            rows[1..].to_vec()
        } else {
            rows.clone()
        };

        // Detect email column
        let header_ref: Vec<String> = headers.clone();
        let email_col = detect_email_column(
            &data_rows,
            if self.skip_header {
                Some(&header_ref)
            } else {
                None
            },
            self.skip_header,
        )
        .ok_or(CsvAnalyzerError::CsvError(CsvErrorType::EmailNotFound))?;

        // Connect to database and get contact properties
        let properties = self.get_contact_properties()?;

        // Detect data types and match field names
        let mut field_names: Vec<String> = Vec::new();
        let mut data_types: Vec<DataType> = Vec::new();
        let mut datetime_format: Option<String> = None;

        for (col_idx, header) in headers.iter().enumerate() {
            self.current_col = col_idx + 1;

            // Get column values
            let col_values: Vec<&str> = data_rows
                .iter()
                .filter_map(|row| row.get(col_idx).map(|s| s.as_str()))
                .collect();

            // Match property
            let matched_prop = match_property(header, &properties);
            let meta_type = matched_prop.map(|p| p.datatype);

            // Detect data type
            let (detected_type, patterns) = detect_data_type(&col_values, meta_type);
            data_types.push(detected_type);

            // Track datetime format
            if detected_type == DataType::DateTime {
                if let Some(ref p) = patterns {
                    if let Some(fmt) = p.format_string() {
                        if datetime_format.is_none()
                            || datetime_format.as_ref().map(|f| f.len()).unwrap_or(0) < fmt.len()
                        {
                            datetime_format = Some(fmt);
                        }
                    }
                }
            }

            // Determine field name
            if col_idx == email_col {
                field_names.push("email".to_string());
            } else if let Some(prop) = matched_prop {
                // Only use property name if types match
                if detected_type == prop.datatype {
                    field_names.push(prop.name.clone());
                } else {
                    field_names.push(String::new());
                }
            } else {
                field_names.push(String::new());
            }
        }

        // Validate field values and prepare data for output
        let mut output_data: Vec<Vec<String>> = Vec::new();
        for (row_idx, row) in data_rows.iter().enumerate() {
            if output_data.len() >= MAX_RETURN_LINES {
                break;
            }

            self.current_row = if self.skip_header {
                row_idx + 2
            } else {
                row_idx + 1
            };

            let mut output_row: Vec<String> = Vec::new();
            for (col_idx, value) in row.iter().enumerate() {
                self.current_col = col_idx + 1;

                if !is_valid_string_size(value) {
                    self.current_field = value.clone();
                    return Err(CsvAnalyzerError::CsvError(CsvErrorType::ValueLong));
                }

                output_row.push(value.clone());
            }
            output_data.push(output_row);
        }

        // Build success response
        let mut response = SuccessResponse::new(&self.config.locale, &self.charset);
        response.skip_header = self.skip_header;
        response.set_field_separator(self.field_delim);
        response.set_text_delimiter(self.text_sep);
        response.date_time_format = datetime_format;
        response.header_names = headers;
        response.field_names = field_names;
        response.data_types = data_types;
        response.data = if output_data.is_empty() {
            None
        } else {
            Some(output_data)
        };

        Ok(response)
    }

    /// Read sample data from file
    fn read_sample(&self) -> Result<Vec<u8>> {
        let file = File::open(&self.config.filename)?;
        let mut reader = BufReader::new(file);
        let mut sample = Vec::new();
        let mut line_count = 0;
        let mut total_bytes = 0;

        loop {
            let mut buffer = vec![0u8; BUFF_SIZE];
            let bytes_read = reader.read(&mut buffer)?;

            if bytes_read == 0 {
                break;
            }

            for &byte in &buffer[..bytes_read] {
                sample.push(byte);
                total_bytes += 1;

                if byte == b'\n' {
                    line_count += 1;
                }

                if line_count > MAX_SCAN_LINES || total_bytes >= MAX_BYTES {
                    break;
                }
            }

            if line_count > MAX_SCAN_LINES || total_bytes >= MAX_BYTES {
                break;
            }
        }

        if sample.is_empty() || line_count == 0 {
            return Err(CsvAnalyzerError::CsvError(CsvErrorType::Sample));
        }

        Ok(sample)
    }

    /// Parse CSV lines into rows
    fn parse_csv(&self, lines: &[&str]) -> Result<Vec<Vec<String>>> {
        let mut rows = Vec::new();

        for line in lines {
            let fields = self.parse_line(line);
            rows.push(fields);
        }

        Ok(rows)
    }

    /// Parse a single CSV line into fields
    fn parse_line(&self, line: &str) -> Vec<String> {
        if self.field_delim == '\0' {
            return vec![line.to_string()];
        }

        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut inside_quotes = false;
        let chars = line.chars().peekable();

        for c in chars {
            if c == self.text_sep {
                inside_quotes = !inside_quotes;
            } else if c == self.field_delim && !inside_quotes {
                fields.push(current_field.trim().to_string());
                current_field = String::new();
            } else {
                current_field.push(c);
            }
        }

        // Don't forget the last field
        fields.push(current_field.trim().to_string());

        fields
    }

    /// Get contact properties from database
    fn get_contact_properties(&self) -> Result<Vec<ContactProperty>> {
        let mut db = DbConnection::new(self.config.db.clone());

        // Try to connect to database
        match db.connect_global() {
            Ok(_) => {}
            Err(e) => {
                // Database not available, return empty properties
                eprintln!("Warning: Database not available: {}", e);
                return Ok(Vec::new());
            }
        }

        // Get pool info
        let pool_info = match db.get_pool_info(self.config.akid) {
            Ok(info) => info,
            Err(e) => {
                eprintln!("Warning: Could not get pool info: {}", e);
                return Ok(Vec::new());
            }
        };

        // Connect to user pool
        if let Err(e) = db.connect_user_pool(&pool_info) {
            eprintln!("Warning: Could not connect to user pool: {}", e);
            return Ok(Vec::new());
        }

        // Get contact properties
        match db.get_contact_properties(self.config.akid) {
            Ok(props) => Ok(props),
            Err(e) => {
                eprintln!("Warning: Could not get contact properties: {}", e);
                Ok(Vec::new())
            }
        }
    }

    /// Build error response
    fn build_error_response(&self, error: CsvAnalyzerError) -> ErrorResponse<'_> {
        let error_type = error.error_type();
        let internal_msg = format!("{}", error);

        ErrorResponse::new(error_type, &self.config.locale, &self.charset)
            .with_internal_message(internal_msg)
            .with_location(self.current_row, self.current_col)
            .with_field(&self.current_field)
            .with_data_type(self.current_data_type)
            .with_column_count(self.current_col_count)
            .with_field_separator(self.field_delim)
            .with_text_delimiter(self.text_sep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbConfig;

    #[test]
    fn test_parse_line_simple() {
        let db_config = DbConfig::new("localhost", 5432, "test", "test", "test");
        let config = Config::new_with_db(1, "en_US", "test.csv", db_config);
        let mut analyzer = CsvAnalyzer::new(config);
        analyzer.field_delim = ',';
        analyzer.text_sep = '"';

        let fields = analyzer.parse_line("a,b,c");
        assert_eq!(fields, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_parse_line_with_quotes() {
        let db_config = DbConfig::new("localhost", 5432, "test", "test", "test");
        let config = Config::new_with_db(1, "en_US", "test.csv", db_config);
        let mut analyzer = CsvAnalyzer::new(config);
        analyzer.field_delim = ',';
        analyzer.text_sep = '"';

        let fields = analyzer.parse_line(r#""hello,world",test,value"#);
        assert_eq!(fields, vec!["hello,world", "test", "value"]);
    }

    #[test]
    fn test_parse_line_semicolon() {
        let db_config = DbConfig::new("localhost", 5432, "test", "test", "test");
        let config = Config::new_with_db(1, "en_US", "test.csv", db_config);
        let mut analyzer = CsvAnalyzer::new(config);
        analyzer.field_delim = ';';
        analyzer.text_sep = '"';

        let fields = analyzer.parse_line("a;b;c");
        assert_eq!(fields, vec!["a", "b", "c"]);
    }
}
