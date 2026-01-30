use crate::types::constants::{TEXT_SEPS, TEXT_SEP_PERCENT};

/// Detect the text/quote separator character.
/// Returns the quote character that appears in pairs across most lines.
pub fn detect_quote_char(lines: &[&str]) -> Option<char> {
    if lines.is_empty() {
        return None;
    }

    // Track stats for each candidate: (char, total_count, lines_with_pairs)
    let mut sep_stats: Vec<(char, usize, usize)> = TEXT_SEPS.iter().map(|&c| (c, 0, 0)).collect();

    for line in lines {
        for stat in sep_stats.iter_mut() {
            let count = line.chars().filter(|&ch| ch == stat.0).count();

            // Must be even (pairs) to be a valid text separator
            if !count.is_multiple_of(2) {
                // Invalid - reset counts for this separator
                stat.1 = 0;
                stat.2 = 0;
                break;
            }

            stat.1 += count;
            if count > 0 {
                stat.2 += 1;
            }
        }
    }

    // Find separator with highest count that appears in enough lines
    sep_stats
        .iter()
        .filter(|&&(_, total, lines_present)| {
            total > 0 && (lines_present * 100 / lines.len()) >= TEXT_SEP_PERCENT
        })
        .max_by_key(|&&(_, total, _)| total)
        .map(|&(sep, _, _)| sep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_double_quote() {
        let lines = vec![
            r#""John","Doe","john@example.com""#,
            r#""Jane","Doe","jane@example.com""#,
        ];
        assert_eq!(detect_quote_char(&lines), Some('"'));
    }

    #[test]
    fn test_detect_single_quote() {
        let lines = vec![
            "'John','Doe','john@example.com'",
            "'Jane','Doe','jane@example.com'",
        ];
        assert_eq!(detect_quote_char(&lines), Some('\''));
    }

    #[test]
    fn test_no_quotes() {
        let lines = vec!["John,Doe,john@example.com", "Jane,Doe,jane@example.com"];
        assert_eq!(detect_quote_char(&lines), None);
    }

    #[test]
    fn test_unbalanced_quotes() {
        // Odd number of quotes - not valid
        let lines = vec![r#""John,Doe,john@example.com"#];
        assert_eq!(detect_quote_char(&lines), None);
    }
}
