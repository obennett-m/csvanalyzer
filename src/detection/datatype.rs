use crate::detection::datetime::{could_be_datetime, guess_datetime_format, DateTimePatterns};
use crate::types::DataType;

/// Tracking state for boolean detection
#[derive(Debug, Clone, Copy, Default)]
pub struct BooleanState {
    /// Whether we've seen string booleans (true/false)
    pub had_string_bool: bool,
}

/// Detect the data type of a single value
pub fn detect_value_type(value: &str, bool_state: &mut BooleanState) -> DataType {
    let value = value.trim();

    // Empty values are strings
    if value.is_empty() {
        return DataType::String;
    }

    // Check if it looks like a string (contains non-numeric chars)
    if !could_be_datetime(value) && is_string_value(value) {
        return DataType::String;
    }

    // Try boolean first (has precedence per MJAPP-2440)
    if let Some(is_string_bool) = try_parse_boolean(value) {
        if is_string_bool {
            bool_state.had_string_bool = true;
        }
        return DataType::Boolean;
    }

    // Try integer
    if try_parse_integer(value) {
        return DataType::Integer;
    }

    // Try float
    if try_parse_float(value) {
        return DataType::Float;
    }

    // Try datetime
    let mut patterns = DateTimePatterns::new();
    if guess_datetime_format(value, &mut patterns) {
        return DataType::DateTime;
    }

    DataType::String
}

/// Detect the data type for an entire column
/// Uses the "downgrading" strategy from Pascal implementation
pub fn detect_data_type(
    values: &[&str],
    meta_type: Option<DataType>,
) -> (DataType, Option<DateTimePatterns>) {
    // If no values but metadata exists, use metadata type
    if values.is_empty() {
        return (meta_type.unwrap_or(DataType::String), None);
    }

    let mut current_type: Option<DataType> = None;
    let mut bool_state = BooleanState::default();
    let mut datetime_patterns: Option<DateTimePatterns> = None;

    for value in values {
        let value = value.trim();

        // Skip empty values
        if value.is_empty() {
            continue;
        }

        let value_type = if could_be_datetime(value) && datetime_patterns.is_some() {
            // For datetime, update the patterns
            let patterns = datetime_patterns.as_mut().unwrap();
            if guess_datetime_format(value, patterns) {
                DataType::DateTime
            } else {
                detect_value_type(value, &mut bool_state)
            }
        } else if could_be_datetime(value) && current_type == Some(DataType::DateTime) {
            // Initialize datetime patterns if first datetime
            let mut patterns = DateTimePatterns::new();
            if guess_datetime_format(value, &mut patterns) {
                datetime_patterns = Some(patterns);
                DataType::DateTime
            } else {
                detect_value_type(value, &mut bool_state)
            }
        } else {
            // Regular type detection, but check datetime specially
            let vt = detect_value_type(value, &mut bool_state);
            if vt == DataType::DateTime {
                let mut patterns = DateTimePatterns::new();
                if guess_datetime_format(value, &mut patterns) {
                    datetime_patterns = Some(patterns);
                }
            }
            vt
        };

        match current_type {
            None => {
                current_type = Some(value_type);
            }
            Some(ct) if ct == value_type => {
                // Same type, continue
            }
            Some(ct) => {
                // Type mismatch - apply downgrading rules
                let new_type = downgrade_types(ct, value_type, &bool_state);
                current_type = Some(new_type);

                // If downgraded to string, we're done
                if new_type == DataType::String {
                    return (DataType::String, None);
                }
            }
        }
    }

    let final_type = current_type.unwrap_or(DataType::String);
    let patterns = if final_type == DataType::DateTime {
        datetime_patterns
    } else {
        None
    };

    (final_type, patterns)
}

/// Downgrade types when there's a mismatch
fn downgrade_types(type1: DataType, type2: DataType, bool_state: &BooleanState) -> DataType {
    use DataType::*;

    match (type1, type2) {
        // Same types
        (a, b) if a == b => a,

        // Integer and Float are compatible -> Float
        (Integer, Float) | (Float, Integer) => Float,

        // Boolean + Integer (when no string bools) -> Integer
        (Boolean, Integer) | (Integer, Boolean) if !bool_state.had_string_bool => Integer,

        // Boolean with string bools + anything else -> String
        (Boolean, _) | (_, Boolean) if bool_state.had_string_bool => String,

        // Otherwise, apply downgrade chain
        _ => {
            let mut t = type1;
            while t != type2 && t != String {
                t = downgrade_one_step(t, bool_state);

                // Check if we reached type2 at intermediate step
                if t == type2 {
                    return t;
                }
            }
            t
        }
    }
}

/// Downgrade a type by one step in the chain
fn downgrade_one_step(t: DataType, bool_state: &BooleanState) -> DataType {
    use DataType::*;

    match t {
        Boolean => {
            if bool_state.had_string_bool {
                String
            } else {
                Integer
            }
        }
        Integer => Float,
        Float => DateTime,
        DateTime => String,
        String => String,
    }
}

/// Check if a value looks like a string (not numeric/boolean/datetime)
fn is_string_value(value: &str) -> bool {
    let value_lower = value.to_lowercase();

    // Check for boolean strings
    if value_lower == "true" || value_lower == "false" {
        return false;
    }

    // Check if all characters are numeric/decimal
    let allowed: &[char] = &[
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ',', '-', '+',
    ];

    for c in value.chars() {
        if !allowed.contains(&c) {
            return true;
        }
    }

    false
}

/// Try to parse as boolean, returns Some(is_string_form) if valid
fn try_parse_boolean(value: &str) -> Option<bool> {
    let value_lower = value.to_lowercase();

    if value_lower == "true" || value_lower == "false" {
        return Some(true); // string form
    }

    if value == "0" || value == "1" {
        return Some(false); // numeric form
    }

    None
}

/// Try to parse as integer
fn try_parse_integer(value: &str) -> bool {
    value.parse::<i64>().is_ok()
}

/// Try to parse as float
fn try_parse_float(value: &str) -> bool {
    // Handle both . and , as decimal separators
    let normalized = value.replace(',', ".");
    normalized.parse::<f64>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_string() {
        let mut bs = BooleanState::default();
        assert_eq!(detect_value_type("hello", &mut bs), DataType::String);
        assert_eq!(detect_value_type("hello world", &mut bs), DataType::String);
    }

    #[test]
    fn test_detect_integer() {
        let mut bs = BooleanState::default();
        assert_eq!(detect_value_type("123", &mut bs), DataType::Integer);
        assert_eq!(detect_value_type("-456", &mut bs), DataType::Integer);
    }

    #[test]
    fn test_detect_float() {
        let mut bs = BooleanState::default();
        assert_eq!(detect_value_type("12.34", &mut bs), DataType::Float);
        assert_eq!(detect_value_type("-56.78", &mut bs), DataType::Float);
    }

    #[test]
    fn test_detect_boolean() {
        let mut bs = BooleanState::default();
        assert_eq!(detect_value_type("true", &mut bs), DataType::Boolean);
        assert!(bs.had_string_bool);

        let mut bs2 = BooleanState::default();
        assert_eq!(detect_value_type("0", &mut bs2), DataType::Boolean);
        assert!(!bs2.had_string_bool);
    }

    #[test]
    fn test_column_type_detection() {
        let values = vec!["1", "2", "3", "4"];
        let (dt, _) = detect_data_type(&values, None);
        assert_eq!(dt, DataType::Integer);
    }

    #[test]
    fn test_column_type_downgrade_to_float() {
        let values = vec!["1", "2", "3.5", "4"];
        let (dt, _) = detect_data_type(&values, None);
        assert_eq!(dt, DataType::Float);
    }

    #[test]
    fn test_column_type_downgrade_to_string() {
        let values = vec!["1", "hello", "3"];
        let (dt, _) = detect_data_type(&values, None);
        assert_eq!(dt, DataType::String);
    }

    #[test]
    fn test_boolean_integer_downgrade() {
        // Boolean (0/1) + larger integer -> Integer
        let values = vec!["0", "1", "5"];
        let (dt, _) = detect_data_type(&values, None);
        assert_eq!(dt, DataType::Integer);
    }

    #[test]
    fn test_string_boolean_integer_downgrade() {
        // Boolean (true/false) + integer -> String
        let values = vec!["true", "false", "5"];
        let (dt, _) = detect_data_type(&values, None);
        assert_eq!(dt, DataType::String);
    }
}
