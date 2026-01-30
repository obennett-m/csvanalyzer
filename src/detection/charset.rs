use crate::types::constants::CSVA_GUESS_SIZE;
use chardetng::EncodingDetector;
use encoding_rs::Encoding;

/// UTF-8 BOM bytes
const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// UTF-16 LE BOM bytes
const UTF16_LE_BOM: &[u8] = &[0xFF, 0xFE];

/// UTF-16 BE BOM bytes
const UTF16_BE_BOM: &[u8] = &[0xFE, 0xFF];

/// Allow guessing UTF-8 encoding
const ALLOW_UTF8: bool = true;

/// Detect the character encoding of the given data.
/// Returns a normalized encoding name.
pub fn detect_charset(data: &[u8]) -> String {
    // Check for BOM markers first
    match data {
        d if d.starts_with(UTF8_BOM) => return "UTF-8BOM".to_string(),
        d if d.len() >= 2 && d.starts_with(UTF16_LE_BOM) => return "UTF-16LE".to_string(),
        d if d.len() >= 2 && d.starts_with(UTF16_BE_BOM) => return "UTF-16BE".to_string(),
        _ => {}
    }

    // For small files, use quick encoding guess
    if data.len() <= CSVA_GUESS_SIZE {
        return guess_encoding_quick(data);
    }

    // Use chardetng for larger files
    let mut detector = EncodingDetector::new();
    detector.feed(data, true);
    let encoding = detector.guess(None, ALLOW_UTF8);

    normalize_encoding(encoding.name())
}

/// Quick encoding detection for small samples
fn guess_encoding_quick(data: &[u8]) -> String {
    // Check if valid UTF-8
    match std::str::from_utf8(data) {
        Ok(_) => "utf8".to_string(),
        Err(_) => {
            // Use chardetng for non-UTF-8
            let mut detector = EncodingDetector::new();
            detector.feed(data, true);
            let encoding = detector.guess(None, ALLOW_UTF8);
            normalize_encoding(encoding.name())
        }
    }
}

/// Normalize encoding name to match Pascal implementation
fn normalize_encoding(name: &str) -> String {
    let name_lower = name.to_lowercase();
    match name_lower.as_str() {
        "utf-8" | "utf8" => "utf8".to_string(),
        "utf-16le" | "utf-16 le" => "UTF-16LE".to_string(),
        "utf-16be" | "utf-16 be" => "UTF-16BE".to_string(),
        "iso-8859-1" | "iso8859-1" | "latin1" => "iso88591".to_string(),
        "iso-8859-15" | "iso8859-15" | "latin9" => "iso885915".to_string(),
        "windows-1252" | "cp1252" => "cp1252".to_string(),
        "windows-1251" | "cp1251" => "cp1251".to_string(),
        _ => name_lower.replace("-", "").replace("_", ""),
    }
}

/// Convert data from detected charset to UTF-8
pub fn convert_to_utf8(data: &[u8], charset: &str) -> Result<String, String> {
    let charset_lower = charset.to_lowercase();

    // Strip BOM if present
    let data = match charset_lower.as_str() {
        "utf-8bom" if data.starts_with(UTF8_BOM) => &data[3..],
        "utf-16le" | "utf16le" if data.len() >= 2 && data.starts_with(UTF16_LE_BOM) => &data[2..],
        "utf-16be" | "utf16be" if data.len() >= 2 && data.starts_with(UTF16_BE_BOM) => &data[2..],
        _ => data,
    };

    // If already UTF-8, just validate and return
    if matches!(charset_lower.as_str(), "utf8" | "utf-8" | "utf-8bom") {
        return String::from_utf8(data.to_vec()).map_err(|e| format!("Invalid UTF-8: {}", e));
    }

    // Get encoding from charset name
    let encoding = match charset_lower.as_str() {
        "utf16le" | "utf-16le" => encoding_rs::UTF_16LE,
        "utf16be" | "utf-16be" => encoding_rs::UTF_16BE,
        "iso88591" | "iso-8859-1" | "latin1" => encoding_rs::WINDOWS_1252, // Superset
        "iso885915" | "iso-8859-15" | "latin9" => encoding_rs::ISO_8859_15,
        "cp1252" | "windows-1252" | "windows1252" => encoding_rs::WINDOWS_1252,
        "cp1251" | "windows-1251" | "windows1251" => encoding_rs::WINDOWS_1251,
        "cp1250" | "windows-1250" | "windows1250" => encoding_rs::WINDOWS_1250,
        // Try to get encoding by name, or fallback to UTF-8
        _ => Encoding::for_label(charset.as_bytes()).unwrap_or(encoding_rs::UTF_8),
    };

    let (decoded, _, had_errors) = encoding.decode(data);
    if had_errors {
        // For now, return the decoded content with replacement chars
        Ok(decoded.into_owned())
    } else {
        Ok(decoded.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_utf8_bom() {
        let data = [0xEF, 0xBB, 0xBF, b'h', b'e', b'l', b'l', b'o'];
        assert_eq!(detect_charset(&data), "UTF-8BOM");
    }

    #[test]
    fn test_detect_utf16_le_bom() {
        let data = [0xFF, 0xFE, b'h', 0, b'i', 0];
        assert_eq!(detect_charset(&data), "UTF-16LE");
    }

    #[test]
    fn test_detect_utf8() {
        let data = b"Hello, World!";
        assert_eq!(detect_charset(data), "utf8");
    }

    #[test]
    fn test_convert_utf8() {
        let data = b"Hello";
        let result = convert_to_utf8(data, "utf8").unwrap();
        assert_eq!(result, "Hello");
    }
}
