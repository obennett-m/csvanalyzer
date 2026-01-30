use once_cell::sync::Lazy;
use regex::Regex;

/// Simple email validation regex (matches FreePascal CheckEmail function behavior)
static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

/// Check if a string is a valid email address
pub fn is_valid_email(email: &str) -> bool {
    let email = email.trim();

    // Must have local part and domain
    let at_pos = match email.find('@') {
        Some(pos) if pos > 0 && pos < email.len() - 1 => pos,
        _ => return false,
    };

    // Domain must have a dot
    if !email[at_pos + 1..].contains('.') {
        return false;
    }

    // Final Boss
    EMAIL_REGEX.is_match(email)
}

/// Detect which column contains email addresses.
/// Returns the column index (0-based) with the most valid emails.
pub fn detect_email_column(
    rows: &[Vec<String>],
    header: Option<&[String]>,
    skip_header: bool,
) -> Option<usize> {
    let num_columns = rows.first()?.len();
    if num_columns == 0 {
        return None;
    }

    // Check header first for "email" or "e-mail"
    if skip_header {
        if let Some(col) = header.and_then(|headers| {
            headers.iter().position(|h| {
                let h_lower = h.to_lowercase();
                matches!(h_lower.as_str(), "email" | "e-mail")
            })
        }) {
            return Some(col);
        }
    }

    // Count valid emails in each column
    let mut email_counts: Vec<usize> = vec![0; num_columns];

    for row in rows {
        for (col_idx, value) in row.iter().enumerate().take(num_columns) {
            if !value.is_empty() && is_valid_email(value) {
                email_counts[col_idx] += 1;
            }
        }
    }

    // Find column with most valid emails
    email_counts
        .iter()
        .enumerate()
        .max_by_key(|&(_, &count)| count)
        .filter(|&(_, &count)| count > 0)
        .map(|(idx, _)| idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(is_valid_email("john@example.com"));
        assert!(is_valid_email("john.doe@example.com"));
        assert!(is_valid_email("john+tag@example.com"));
        assert!(is_valid_email("john@sub.example.com"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!is_valid_email(""));
        assert!(!is_valid_email("john"));
        assert!(!is_valid_email("john@"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("john@example"));
    }

    #[test]
    fn test_detect_email_column_by_header() {
        let rows = vec![
            vec!["John".to_string(), "john@example.com".to_string()],
            vec!["Jane".to_string(), "jane@example.com".to_string()],
        ];
        let header = vec!["name".to_string(), "email".to_string()];
        assert_eq!(detect_email_column(&rows, Some(&header), true), Some(1));
    }

    #[test]
    fn test_detect_email_column_by_content() {
        let rows = vec![
            vec!["John".to_string(), "john@example.com".to_string()],
            vec!["Jane".to_string(), "jane@example.com".to_string()],
        ];
        assert_eq!(detect_email_column(&rows, None, false), Some(1));
    }

    #[test]
    fn test_detect_email_column_first_col() {
        let rows = vec![
            vec!["john@example.com".to_string(), "John".to_string()],
            vec!["jane@example.com".to_string(), "Jane".to_string()],
        ];
        assert_eq!(detect_email_column(&rows, None, false), Some(0));
    }
}
