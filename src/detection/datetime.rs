use crate::types::{DATE_PATTERNS, TIME_PATTERNS};
use chrono::NaiveDateTime;

/// Date separator characters
const DATE_SEPS: &[char] = &['/', '-', '.'];

/// Time separator characters
const TIME_SEPS: &[char] = &[':'];

/// Minimum length of a RFC3339 date including time character
/// e.g. 2020-01-01T
const MIN_RFC3339_LEN: usize = 11;

/// Patterns that have been validated for a column
#[derive(Debug, Clone, Default)]
pub struct DateTimePatterns {
    pub date_patterns: Vec<(String, char)>, // (pattern, separator)
    pub time_patterns: Vec<(String, char)>, // (pattern, separator)
}

impl DateTimePatterns {
    pub fn new() -> Self {
        let mut patterns = DateTimePatterns {
            date_patterns: Vec::new(),
            time_patterns: Vec::new(),
        };

        // Add RFC3339 first
        patterns.date_patterns.push(("rfc3339".to_string(), '-'));

        // Add standard date patterns
        for dp in DATE_PATTERNS {
            patterns
                .date_patterns
                .push((dp.pattern.to_string(), dp.separator));
        }

        // Add time patterns
        for tp in TIME_PATTERNS {
            patterns
                .time_patterns
                .push((tp.pattern.to_string(), tp.separator));
        }

        patterns
    }

    /// Get the best date pattern (first remaining)
    pub fn best_date_pattern(&self) -> Option<&str> {
        self.date_patterns.first().map(|(p, _)| p.as_str())
    }

    /// Get the best time pattern (first remaining)
    pub fn best_time_pattern(&self) -> Option<&str> {
        self.time_patterns.first().map(|(p, _)| p.as_str())
    }

    /// Get combined datetime format string
    pub fn format_string(&self) -> Option<String> {
        match (self.best_date_pattern(), self.best_time_pattern()) {
            (Some(date), Some(time)) => Some(format!("{} {}", date, time)),
            (Some(date), None) => Some(date.to_string()),
            _ => None,
        }
    }
}

/// Check if a string could potentially be a datetime value
pub fn could_be_datetime(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    // Must contain date/time separator characters and digits
    let has_date_sep = value.chars().any(|c| DATE_SEPS.contains(&c));
    let has_time_sep = value.chars().any(|c| TIME_SEPS.contains(&c));
    let has_digits = value.chars().any(|c| c.is_ascii_digit());

    // Must have digits and at least one separator type
    has_digits && (has_date_sep || has_time_sep)
}

/// Attempt to guess the datetime format of a value.
/// Updates the pattern lists by removing non-matching patterns.
/// Returns true if the value matches at least one remaining pattern.
pub fn guess_datetime_format(value: &str, patterns: &mut DateTimePatterns) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return !patterns.date_patterns.is_empty();
    }

    // Detect separators in the value
    let date_sep = match value.chars().find(|c| DATE_SEPS.contains(c)) {
        Some(s) => s,
        None => {
            patterns.date_patterns.clear();
            patterns.time_patterns.clear();
            return false;
        }
    };

    // Filter date patterns by separator
    patterns
        .date_patterns
        .retain(|(pattern, sep)| *sep == date_sep || pattern == "rfc3339");

    // Check RFC3339 format
    if patterns.date_patterns.iter().any(|(p, _)| p == "rfc3339") {
        if is_rfc3339(value) {
            patterns.date_patterns.retain(|(p, _)| p == "rfc3339");
            patterns.time_patterns.clear();
            return true;
        }
        patterns.date_patterns.retain(|(p, _)| p != "rfc3339");
    }

    // Try to parse with each remaining date pattern
    let time_sep = value.chars().find(|c| TIME_SEPS.contains(c));
    patterns
        .date_patterns
        .retain(|(pattern, sep)| try_parse_date(value, pattern, *sep, time_sep));

    // Validate time patterns if time separator exists
    if let Some(ts) = time_sep {
        patterns
            .time_patterns
            .retain(|(pattern, sep)| *sep == ts && try_parse_time(value, pattern));
    } else {
        // No time component
        patterns.time_patterns.clear();
    }

    !patterns.date_patterns.is_empty()
}

/// Check if value is RFC3339 format
fn is_rfc3339(value: &str) -> bool {
    value.len() >= MIN_RFC3339_LEN
        && matches!(value.chars().nth(10), Some('T') | Some('t'))
        && (chrono::DateTime::parse_from_rfc3339(value).is_ok()
            || chrono::DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.fZ").is_ok()
            || chrono::DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%SZ").is_ok())
}

/// Try to parse a date value with a specific pattern
fn try_parse_date(value: &str, pattern: &str, date_sep: char, time_sep: Option<char>) -> bool {
    let date_part = if time_sep.is_some() {
        value.split_whitespace().next().unwrap_or(value)
    } else {
        value
    };

    let chrono_pattern = pattern_to_chrono(pattern, date_sep);

    NaiveDateTime::parse_from_str(
        &format!("{} 00:00:00", date_part),
        &format!("{} %H:%M:%S", chrono_pattern),
    )
    .is_ok()
        || chrono::NaiveDate::parse_from_str(date_part, &chrono_pattern).is_ok()
}

/// Try to parse a time value with a specific pattern
fn try_parse_time(value: &str, pattern: &str) -> bool {
    value
        .split_whitespace()
        .nth(1)
        .map(|time_part| {
            let chrono_pattern = time_pattern_to_chrono(pattern);
            chrono::NaiveTime::parse_from_str(time_part, &chrono_pattern).is_ok()
        })
        .unwrap_or(false)
}

/// Convert Pascal date pattern to chrono format
fn pattern_to_chrono(pattern: &str, sep: char) -> String {
    pattern
        .replace("yyyy", "%Y")
        .replace("mm", "%m")
        .replace("dd", "%d")
        .replace('-', &sep.to_string())
        .replace('/', &sep.to_string())
        .replace('.', &sep.to_string())
}

/// Convert Pascal time pattern to chrono format
fn time_pattern_to_chrono(pattern: &str) -> String {
    let p = pattern
        .replace("hh", "%H")
        .replace("nn", "%M")
        .replace("ss", "%S");

    // Handle am/pm
    if pattern.contains("am/pm") {
        p.replace(" am/pm", " %p").replace("%H", "%I")
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_could_be_datetime() {
        assert!(could_be_datetime("2020-01-15"));
        assert!(could_be_datetime("15/01/2020"));
        assert!(could_be_datetime("10:30:00"));
        assert!(could_be_datetime("2020-01-15 10:30:00"));
        assert!(!could_be_datetime("hello"));
        assert!(!could_be_datetime("12345"));
    }

    #[test]
    fn test_is_rfc3339() {
        assert!(is_rfc3339("2020-01-15T10:30:00Z"));
        assert!(!is_rfc3339("2020-01-15"));
        assert!(!is_rfc3339("2020-01-15 10:30:00"));
    }

    #[test]
    fn test_guess_datetime_format() {
        let mut patterns = DateTimePatterns::new();
        assert!(guess_datetime_format("2020-01-15", &mut patterns));
        assert!(patterns
            .date_patterns
            .iter()
            .any(|(p, _)| p == "yyyy-mm-dd"));
    }

    #[test]
    fn test_guess_datetime_with_time() {
        let mut patterns = DateTimePatterns::new();
        assert!(guess_datetime_format("2020-01-15 10:30:00", &mut patterns));
        assert!(patterns.best_date_pattern().is_some());
    }
}
