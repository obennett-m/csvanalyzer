use crate::detection::email::is_valid_email;
use crate::types::constants::{EMAIL_DOMAIN_CHARS, EMAIL_LOCAL_CHARS, FIELD_DELIMS, FIELD_DELIM_PERCENT};

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
    let len = chars.len();

    let mut pos = 0;
    while pos < len {
        // Find @ sign
        if chars[pos] != '@' {
            pos += 1;
            continue;
        }

        let at_pos = pos;

        // Find start of local part
        let mut local_start = at_pos;
        for i in (0..at_pos).rev() {
            let c = chars[i].to_ascii_lowercase();
            if !EMAIL_LOCAL_CHARS.contains(c) {
                local_start = i + 1;
                break;
            }
            if i == 0 {
                local_start = 0;
            }
        }

        // Find end of domain
        let mut domain_end = at_pos;
        for i in (at_pos + 1)..len {
            let c = chars[i].to_ascii_lowercase();
            if !EMAIL_DOMAIN_CHARS.contains(c) {
                domain_end = i - 1;
                break;
            }
            if i == len - 1 {
                domain_end = len - 1;
            }
        }

        // Extract potential email
        let local_part: String = chars[local_start..at_pos].iter().collect();
        let domain: String = chars[(at_pos + 1)..=domain_end].iter().collect();
        let email = format!("{}@{}", local_part.to_lowercase(), domain.to_lowercase());

        if is_valid_email(&email) {
            // Look for delimiter after domain
            let mut right_delim = None;
            let mut i = domain_end + 1;
            while i < len {
                let c = chars[i];
                if c != ' ' {
                    if is_field_delimiter(c) {
                        right_delim = Some(c);
                    }
                    break;
                }
                i += 1;
            }

            // Look for delimiter before local part
            let mut left_delim = None;
            if local_start > 0 {
                let mut i = local_start - 1;
                loop {
                    let c = chars[i];
                    if c != ' ' {
                        if is_field_delimiter(c) {
                            left_delim = Some(c);
                        }
                        break;
                    }
                    if i == 0 {
                        break;
                    }
                    i -= 1;
                }
            }

            // Choose delimiter based on priority
            if let (Some(l), Some(r)) = (left_delim, right_delim) {
                if l == r {
                    return Some(l);
                }
                return Some(get_priority_delimiter(l, r));
            } else if let Some(r) = right_delim {
                return Some(r);
            } else if let Some(l) = left_delim {
                return Some(l);
            }
        }

        pos = at_pos + 1;
    }

    None
}

/// Check if character is a valid field delimiter
fn is_field_delimiter(c: char) -> bool {
    FIELD_DELIMS.contains(&c)
}

/// Get the delimiter with higher priority
fn get_priority_delimiter(c1: char, c2: char) -> char {
    for d in FIELD_DELIMS.iter() {
        if *d == c1 || *d == c2 {
            return *d;
        }
    }
    c1
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

    // Find delimiter with highest count
    let mut best_idx: Option<usize> = None;
    for (i, stat) in delim_stats.iter().enumerate() {
        if stat.1 > 0 && (best_idx.is_none() || stat.1 > delim_stats[best_idx.unwrap()].1) {
            best_idx = Some(i);
        }
    }

    // Check if winner exists in at least FIELD_DELIM_PERCENT of lines
    if let Some(idx) = best_idx {
        let percentage = (delim_stats[idx].2 * 100) / lines.len();
        if percentage >= FIELD_DELIM_PERCENT {
            return Some(delim_stats[idx].0);
        }
    }

    None
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
