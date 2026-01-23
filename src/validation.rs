use crate::detection::delimiter::count_delimiters;
use crate::error::{CsvAnalyzerError, Result};
use crate::types::constants::{COLUMN_COUNT_PERCENT, MAX_BUCKET, MAX_COLUMNS, MAX_STRING_SIZE};
use crate::types::CsvErrorType;
use std::collections::HashMap;

/// Validation results
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub columns_count: usize,
    pub error_row: usize,
}

/// Validate that column counts are consistent across lines.
/// Returns the dominant column count if valid.
pub fn validate_columns_count(
    lines: &[&str],
    delimiter: char,
    text_sep: char,
) -> Result<ValidationResult> {
    if lines.is_empty() {
        return Err(CsvAnalyzerError::CsvError(CsvErrorType::Sample));
    }

    // No delimiter means single column
    if delimiter == '\0' {
        return Ok(ValidationResult {
            columns_count: 1,
            error_row: 0,
        });
    }

    // Count columns in each line (bucket approach like Pascal)
    let mut bucket: HashMap<usize, usize> = HashMap::new();

    for line in lines.iter() {
        // +1 because N delimiters = N+1 columns
        let col_count = count_delimiters(delimiter, line, text_sep) + 1;

        let entry = bucket.entry(col_count).or_insert(0);
        *entry += 1;

        // Too many different column counts
        if bucket.len() > MAX_BUCKET {
            return Err(CsvAnalyzerError::CsvError(CsvErrorType::VariousFieldsCount));
        }
    }

    // Find the dominant column count (must be >= COLUMN_COUNT_PERCENT)
    let total_lines = lines.len();
    for (&col_count, &occurrence) in bucket.iter() {
        let percentage = (occurrence * 100) / total_lines;
        if percentage >= COLUMN_COUNT_PERCENT {
            // Check max columns
            if col_count > MAX_COLUMNS {
                return Err(CsvAnalyzerError::CsvError(CsvErrorType::TooMuchColumns));
            }

            return Ok(ValidationResult {
                columns_count: col_count,
                error_row: 0,
            });
        }
    }

    Err(CsvAnalyzerError::CsvError(CsvErrorType::VariousFieldsCount))
}

/// Check if a string exceeds max length
pub fn is_valid_string_size(s: &str) -> bool {
    s.len() <= MAX_STRING_SIZE
}

/// Validate a column name
pub fn validate_column_name(name: &str, _col_idx: usize) -> Result<()> {
    if !is_valid_string_size(name) {
        return Err(CsvAnalyzerError::CsvError(CsvErrorType::ColumnLong));
    }
    Ok(())
}

/// Validate a field value
pub fn validate_field_value(value: &str, _row_idx: usize, _col_idx: usize) -> Result<()> {
    if !is_valid_string_size(value) {
        return Err(CsvAnalyzerError::CsvError(CsvErrorType::ValueLong));
    }
    Ok(())
}

/// Check for duplicate field names in header
pub fn check_duplicate_fields(headers: &[String]) -> Result<()> {
    let mut seen: HashMap<String, usize> = HashMap::new();

    for (idx, header) in headers.iter().enumerate() {
        let lower = header.to_lowercase();
        if let Some(_prev_idx) = seen.get(&lower) {
            return Err(CsvAnalyzerError::CsvError(CsvErrorType::DuplicateField));
        }
        seen.insert(lower, idx);
    }

    Ok(())
}

/// Check if sample data appears to be binary
pub fn is_binary_data(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    // Check first line only
    let first_line_end = data
        .iter()
        .position(|&b| b == b'\n')
        .unwrap_or(data.len())
        .min(1024);

    let sample = &data[..first_line_end];

    // Skip BOM checks
    let sample = if sample.len() >= 3 && sample.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &sample[3..]
    } else if sample.len() >= 2
        && (sample.starts_with(&[0xFF, 0xFE]) || sample.starts_with(&[0xFE, 0xFF]))
    {
        &sample[2..]
    } else {
        sample
    };

    if sample.is_empty() {
        return false;
    }

    // Count unprintable characters
    let unprintable_count = sample
        .iter()
        .filter(|&&b| b < 0x20 || b == 0xFF || (b >= 0x7F && b <= 0xA0))
        .count();

    // If more than 20% unprintable, consider binary
    let percentage = (unprintable_count * 100) / sample.len();
    percentage >= 20
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_consistent_columns() {
        let lines = vec!["a,b,c", "1,2,3", "x,y,z"];
        let result = validate_columns_count(&lines, ',', '"').unwrap();
        assert_eq!(result.columns_count, 3);
    }

    #[test]
    fn test_validate_inconsistent_columns() {
        let lines = vec!["a,b,c", "1,2", "x,y,z,w"];
        // With 3 different counts and only 1 occurrence each, no majority
        let result = validate_columns_count(&lines, ',', '"');
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_string_size() {
        assert!(is_valid_string_size("hello"));
        let long_string = "x".repeat(MAX_STRING_SIZE + 1);
        assert!(!is_valid_string_size(&long_string));
    }

    #[test]
    fn test_check_duplicate_fields() {
        let headers = vec![
            "name".to_string(),
            "email".to_string(),
            "country".to_string(),
        ];
        assert!(check_duplicate_fields(&headers).is_ok());

        let dup_headers = vec!["name".to_string(), "email".to_string(), "Name".to_string()];
        assert!(check_duplicate_fields(&dup_headers).is_err());
    }

    #[test]
    fn test_is_binary_data() {
        let text = b"hello,world\n";
        assert!(!is_binary_data(text));

        let binary = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        assert!(is_binary_data(&binary));
    }
}
