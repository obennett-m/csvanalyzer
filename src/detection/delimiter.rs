use crate::detection::email::is_valid_email;
use crate::types::constants::{
    EMAIL_DOMAIN_CHARS, EMAIL_LOCAL_CHARS, FIELD_DELIMS, FIELD_DELIM_PERCENT,
};

/// Detect the field delimiter in CSV lines.
/// Uses email-based detection as primary method, falling back to frequency-based.
pub fn detect_delimiter(lines: &[&str], text_sep: Option<char>) -> Option<char> {
    let text_sep = text_sep.unwrap_or('\0');

    // Primary: Email-based detection
    if let Some(delim) = detect_delimiter_from_email(lines) {
        return Some(delim);
    }

    // Fallback: Frequency-based detection
    detect_delimiter_by_frequency(lines, text_sep)
}

/// Detect delimiter based on characters adjacent to email addresses
fn detect_delimiter_from_email(lines: &[&str]) -> Option<char> {
    for line in lines {
        if let Some(delim) = get_email_delimiter(line) {
            return Some(delim);
        }
    }
    None
}

/// Find delimiter adjacent to email in a line
fn get_email_delimiter(line: &str) -> Option<char> {
    let chars: Vec<char> = line.chars().collect();

    for (at_pos, &ch) in chars.iter().enumerate() {
        // Find @ sign
        if ch != '@' {
            continue;
        }

        // Find start of local part
        let local_start = chars[..at_pos]
            .iter()
            .rposition(|&c| !EMAIL_LOCAL_CHARS.contains(c.to_ascii_lowercase()))
            .map(|pos| pos + 1)
            .unwrap_or(0);

        // Find end of domain
        let domain_end = chars[(at_pos + 1)..]
            .iter()
            .position(|&c| !EMAIL_DOMAIN_CHARS.contains(c.to_ascii_lowercase()))
            .map(|pos| at_pos + pos)
            .unwrap_or(chars.len() - 1);

        // Extract potential email
        let local_part: String = chars[local_start..at_pos].iter().collect();
        let domain: String = chars[(at_pos + 1)..=domain_end].iter().collect();
        let email = format!("{}@{}", local_part.to_lowercase(), domain.to_lowercase());

        if !is_valid_email(&email) {
            continue;
        }

        // Look for delimiters around email
        let right_delim = chars[(domain_end + 1)..]
            .iter()
            .find(|&&c| c != ' ')
            .filter(|&&c| is_field_delimiter(c))
            .copied();

        let left_delim = if local_start > 0 {
            chars[..local_start]
                .iter()
                .rev()
                .find(|&&c| c != ' ')
                .filter(|&&c| is_field_delimiter(c))
                .copied()
        } else {
            None
        };

        // Choose delimiter based on priority
        return match (left_delim, right_delim) {
            (Some(l), Some(r)) if l == r => Some(l),
            (Some(l), Some(r)) => Some(get_priority_delimiter(l, r)),
            (None, Some(r)) => Some(r),
            (Some(l), None) => Some(l),
            _ => None,
        };
    }

    None
}

/// Check if character is a valid field delimiter
fn is_field_delimiter(c: char) -> bool {
    FIELD_DELIMS.contains(&c)
}

/// Get the delimiter with higher priority
fn get_priority_delimiter(c1: char, c2: char) -> char {
    FIELD_DELIMS
        .iter()
        .find(|&&d| d == c1 || d == c2)
        .copied()
        .unwrap_or(c1)
}

/// Detect delimiter by counting frequencies across lines
fn detect_delimiter_by_frequency(lines: &[&str], text_sep: char) -> Option<char> {
    if lines.is_empty() {
        return None;
    }

    let mut delim_stats: Vec<(char, usize, usize)> = FIELD_DELIMS
        .iter()
        .map(|&d| (d, 0, 0)) // (delimiter, total_count, lines_present)
        .collect();

    for line in lines {
        for stat in delim_stats.iter_mut() {
            let count = count_delimiters(stat.0, line, text_sep);
            stat.1 += count;
            if count > 0 {
                stat.2 += 1;
            }
        }
    }

    // Find delimiter with highest count that appears in enough lines
    delim_stats
        .iter()
        .filter(|&&(_, total, lines_present)| {
            total > 0 && (lines_present * 100 / lines.len()) >= FIELD_DELIM_PERCENT
        })
        .max_by_key(|&&(_, total, _)| total)
        .map(|&(delim, _, _)| delim)
}

/// Count occurrences of a delimiter in a line, respecting text separators
pub fn count_delimiters(delimiter: char, line: &str, text_sep: char) -> usize {
    let mut count = 0;
    let mut inside_text = false;

    for c in line.chars() {
        if text_sep != '\0' && c == text_sep {
            inside_text = !inside_text;
        }

        if c == delimiter && !inside_text {
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_comma_delimiter() {
        let lines = vec!["john@example.com,John,Doe", "jane@example.com,Jane,Doe"];
        assert_eq!(detect_delimiter(&lines, None), Some(','));
    }

    #[test]
    fn test_detect_semicolon_delimiter() {
        let lines = vec!["john@example.com;John;Doe", "jane@example.com;Jane;Doe"];
        assert_eq!(detect_delimiter(&lines, None), Some(';'));
    }

    #[test]
    fn test_detect_tab_delimiter() {
        let lines = vec!["john@example.com\tJohn\tDoe", "jane@example.com\tJane\tDoe"];
        assert_eq!(detect_delimiter(&lines, None), Some('\t'));
    }

    #[test]
    fn test_count_delimiters_with_quotes() {
        let line = r#""hello,world",test,value"#;
        assert_eq!(count_delimiters(',', line, '"'), 2);
    }

    #[test]
    fn test_count_delimiters_without_quotes() {
        let line = "hello,world,test";
        assert_eq!(count_delimiters(',', line, '\0'), 2);
    }
}
