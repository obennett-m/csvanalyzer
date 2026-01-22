use once_cell::sync::Lazy;
use regex::Regex;

/// Simple email validation regex (matches Pascal CheckEmail function behavior)
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

/// Check if a string is a valid email address
pub fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    if email.is_empty() {
        return false;
    }

    // Basic structural checks
    let at_pos = match email.find('@') {
        Some(pos) => pos,
        None => return false,
    };

    // Must have local part and domain
    if at_pos == 0 || at_pos == email.len() - 1 {
        return false;
    }

    // Domain must have a dot
    let domain = &email[at_pos + 1..];
    if !domain.contains('.') {
        return false;
    }

    // Use regex for final validation
    EMAIL_REGEX.is_match(email)
}

/// Detect which column contains email addresses.
/// Returns the column index (0-based) with the most valid emails.
pub fn detect_email_column(
    rows: &[Vec<String>],
    header: Option<&[String]>,
    skip_header: bool,
) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }

    let num_columns = rows[0].len();
    if num_columns == 0 {
        return None;
    }

    // First check if header has "email" or "e-mail"
    if skip_header {
        if let Some(headers) = header {
            for (i, h) in headers.iter().enumerate() {
                let h_lower = h.to_lowercase();
                if h_lower == "email" || h_lower == "e-mail" {
                    return Some(i);
                }
            }
        }
    }

    // Count valid emails in each column
    let mut email_counts: Vec<usize> = vec![0; num_columns];

    for row in rows {
        for (col_idx, value) in row.iter().enumerate() {
            if col_idx < email_counts.len() && !value.is_empty() && is_valid_email(value) {
                email_counts[col_idx] += 1;
            }
        }
    }

    // Find column with most valid emails
    let mut best_col: Option<usize> = None;
    let mut best_count = 0;

    for (col_idx, &count) in email_counts.iter().enumerate() {
        if count > best_count {
            best_count = count;
            best_col = Some(col_idx);
        }
    }

    // If not found by counting, check header again for "e-mail" variant
    if best_col.is_none() && skip_header {
        if let Some(headers) = header {
            for (i, h) in headers.iter().enumerate() {
                let h_lower = h.to_lowercase();
                if h_lower == "e-mail" {
                    return Some(i);
                }
            }
        }
    }

    best_col
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
