use crate::detection::email::is_valid_email;
use crate::types::constants::{EMAIL_DOMAIN_CHARS, EMAIL_LOCAL_CHARS};

/// Detect whether the first line is a header.
/// Returns true if the first line appears to be a header (not data).
pub fn has_header(lines: &[&str], text_sep: char, delimiter: char) -> bool {
    let first_line = match lines.first() {
        Some(&line) => line,
        None => return true,
    };

    // Check if first line contains a valid email address - if so, no header
    if contains_valid_email(first_line) {
        return false;
    }

    // If header has empty fields (e.g., ";;;"), consider it not a header
    let has_content = first_line
        .chars()
        .any(|c| c != delimiter && c != text_sep && !c.is_whitespace());

    // Default: assume has header
    has_content
}

/// Check if a line contains a valid email address
fn contains_valid_email(line: &str) -> bool {
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

        // Extract and validate email
        if local_start < at_pos && at_pos < domain_end {
            let local_part: String = chars[local_start..at_pos].iter().collect();
            let domain: String = chars[(at_pos + 1)..=domain_end].iter().collect();
            let email = format!("{}@{}", local_part.to_lowercase(), domain.to_lowercase());

            if is_valid_email(&email) {
                return true;
            }
        }
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
        assert!(contains_valid_email("john+lol1@example.com,John"));
        assert!(!contains_valid_email("email,name,country"));
    }
}
