use crate::detection::email::is_valid_email;
use crate::types::constants::{EMAIL_DOMAIN_CHARS, EMAIL_LOCAL_CHARS};

/// Detect whether the first line is a header.
/// Returns true if the first line appears to be a header (not data).
pub fn has_header(lines: &[&str], text_sep: char, delimiter: char) -> bool {
    if lines.is_empty() {
        return true;
    }

    let first_line = lines[0];

    // Check if first line contains a valid email address - if so, no header
    if contains_valid_email(first_line) {
        return false;
    }

    // If header has empty fields (e.g., ";;;"), consider it not a header
    let stripped: String = first_line
        .chars()
        .filter(|&c| c != delimiter && c != text_sep)
        .collect();

    if stripped.trim().is_empty() {
        return false;
    }

    // Default: assume has header
    true
}

/// Check if a line contains a valid email address
fn contains_valid_email(line: &str) -> bool {
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

        // Extract and validate email
        if local_start < at_pos && at_pos < domain_end {
            let local_part: String = chars[local_start..at_pos].iter().collect();
            let domain: String = chars[(at_pos + 1)..=domain_end].iter().collect();
            let email = format!("{}@{}", local_part.to_lowercase(), domain.to_lowercase());

            if is_valid_email(&email) {
                return true;
            }
        }

        pos = at_pos + 1;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_no_email() {
        let lines = vec!["email,name,country", "john@example.com,John,US"];
        assert!(has_header(&lines, '"', ','));
    }

    #[test]
    fn test_no_header_has_email() {
        let lines = vec!["john@example.com,John,US", "jane@example.com,Jane,UK"];
        assert!(!has_header(&lines, '"', ','));
    }

    #[test]
    fn test_empty_header() {
        let lines = vec![";;;", "john@example.com;John;US"];
        assert!(!has_header(&lines, '"', ';'));
    }

    #[test]
    fn test_contains_valid_email() {
        assert!(contains_valid_email("john@example.com,John"));
        assert!(!contains_valid_email("email,name,country"));
    }
}
