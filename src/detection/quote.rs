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
            let count = count_char(stat.0, line);

            // Must be even (pairs) to be a valid text separator
            if count % 2 != 0 {
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

    // Find separator with highest count
    let mut best_idx: Option<usize> = None;
    for (i, stat) in sep_stats.iter().enumerate() {
        if stat.1 > 0 && (best_idx.is_none() || stat.1 > sep_stats[best_idx.unwrap()].1) {
            best_idx = Some(i);
        }
    }

    // Check if winner exists in at least TEXT_SEP_PERCENT of lines
    if let Some(idx) = best_idx {
        let percentage = (sep_stats[idx].2 * 100) / lines.len();
        if percentage >= TEXT_SEP_PERCENT {
            return Some(sep_stats[idx].0);
        }
    }

    None
}

/// Count occurrences of a character in a string
fn count_char(c: char, s: &str) -> usize {
    s.chars().filter(|&ch| ch == c).count()
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
