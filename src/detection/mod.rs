pub mod charset;
pub mod datatype;
pub mod datetime;
pub mod delimiter;
pub mod email;
pub mod header;
pub mod quote;

pub use charset::detect_charset;
pub use datatype::detect_data_type;
pub use datetime::{could_be_datetime, guess_datetime_format, DateTimePatterns};
pub use delimiter::detect_delimiter;
pub use email::detect_email_column;
pub use header::has_header;
pub use quote::detect_quote_char;
