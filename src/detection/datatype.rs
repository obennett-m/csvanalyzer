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

    if value.is_empty() {
        return DataType::String;
    }

    // Check if it looks like a string (contains non-numeric chars)
    if !could_be_datetime(value) && is_string_value(value) {
        return DataType::String;
    }

    // Try boolean first (has precedence per MJAPP-2440)
    if let Some(is_string_bool) = try_parse_boolean(value) {
        bool_state.had_string_bool |= is_string_bool;
        return DataType::Boolean;
    }

    // Try integer -> float -> datetime -> string
    if try_parse_integer(value) {
        DataType::Integer
    } else if try_parse_float(value) {
        DataType::Float
    } else if guess_datetime_format(value, &mut DateTimePatterns::new()) {
        DataType::DateTime
    } else {
        DataType::String
    }
}

/// Detect the data type for an entire column
/// Uses the "downgrading" strategy from Pascal implementation
pub fn detect_data_type(
    values: &[&str],
    meta_type: Option<DataType>,
) -> (DataType, Option<DateTimePatterns>) {
    if values.is_empty() {
        return (meta_type.unwrap_or(DataType::String), None);
    }

    let mut current_type: Option<DataType> = None;
    let mut bool_state = BooleanState::default();
    let mut datetime_patterns: Option<DateTimePatterns> = None;

    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }

        let value_type = detect_value_with_patterns(value, &mut bool_state, &mut datetime_patterns);

        current_type = match current_type {
            None => Some(value_type),
            Some(ct) if ct == value_type => Some(ct),
            Some(ct) => {
                let new_type = downgrade_types(ct, value_type, &bool_state);
                if new_type == DataType::String {
                    return (DataType::String, None);
                }
                Some(new_type)
            }
        };
    }

    let final_type = current_type.unwrap_or(DataType::String);
    let patterns = if final_type == DataType::DateTime {
        datetime_patterns
    } else {
        None
    };

    (final_type, patterns)
}

/// Helper to detect value type while managing datetime patterns
fn detect_value_with_patterns(
    value: &str,
    bool_state: &mut BooleanState,
    datetime_patterns: &mut Option<DateTimePatterns>,
) -> DataType {
    if !could_be_datetime(value) {
        let vt = detect_value_type(value, bool_state);
        if vt == DataType::DateTime {
            let mut patterns = DateTimePatterns::new();
            if guess_datetime_format(value, &mut patterns) {
                *datetime_patterns = Some(patterns);
            }
        }
        return vt;
    }

    // Try to use or initialize datetime patterns
    if let Some(patterns) = datetime_patterns {
        if guess_datetime_format(value, patterns) {
            return DataType::DateTime;
        }
    } else {
        let mut patterns = DateTimePatterns::new();
        if guess_datetime_format(value, &mut patterns) {
            *datetime_patterns = Some(patterns);
            return DataType::DateTime;
        }
    }

    detect_value_type(value, bool_state)
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
        // Otherwise, downgrade until compatible or String
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
        Boolean if bool_state.had_string_bool => String,
        Boolean => Integer,
        Integer => Float,
        Float => DateTime,
        DateTime | String => String,
    }
}

/// Check if a value looks like a string (not numeric/boolean/datetime)
fn is_string_value(value: &str) -> bool {
    let value_lower = value.to_lowercase();

    // Check for boolean strings
    if matches!(value_lower.as_str(), "true" | "false") {
        return false;
    }

    // Check if contains non-numeric characters
    value
        .chars()
        .any(|c| !matches!(c, '0'..='9' | '.' | ',' | '-' | '+'))
}

/// Try to parse as boolean, returns Some(is_string_form) if valid
fn try_parse_boolean(value: &str) -> Option<bool> {
    let value_lower = value.to_lowercase();
    match value_lower.as_str() {
        "true" | "false" => Some(true),                 // string form
        _ if matches!(value, "0" | "1") => Some(false), // numeric form
        _ => None,
    }
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
