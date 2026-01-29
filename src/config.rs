use crate::error::{CsvAnalyzerError, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
}

impl DbConfig {
    /// Create a new DbConfig with explicit values
    pub fn new(host: String, port: u16, database: String, user: String, password: String) -> Self {
        DbConfig {
            host,
            port,
            database,
            user,
            password,
        }
    }

    /// Load configuration from environment variables
    /// Returns an error if required environment variables are not set
    pub fn from_env() -> Result<Self> {
        let host = env::var("PGHOST").map_err(|_| {
            CsvAnalyzerError::ConfigError("PGHOST environment variable not set".to_string())
        })?;

        let port = env::var("PGPORT")
            .map_err(|_| {
                CsvAnalyzerError::ConfigError("PGPORT environment variable not set".to_string())
            })?
            .parse()
            .map_err(|_| {
                CsvAnalyzerError::ConfigError("PGPORT must be a valid port number".to_string())
            })?;

        let database = env::var("PGDATABASE").map_err(|_| {
            CsvAnalyzerError::ConfigError("PGDATABASE environment variable not set".to_string())
        })?;

        let user = env::var("PGUSER").map_err(|_| {
            CsvAnalyzerError::ConfigError("PGUSER environment variable not set".to_string())
        })?;

        let password = env::var("PGPASSWORD").map_err(|_| {
            CsvAnalyzerError::ConfigError("PGPASSWORD environment variable not set".to_string())
        })?;

        Ok(DbConfig {
            host,
            port,
            database,
            user,
            password,
        })
    }

    /// Load configuration from a config file (mailjet.conf format)
    /// Reads from the [PGGLOBAL] section
    /// Returns an error if required configuration values are missing
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            CsvAnalyzerError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // Check if this is a section header
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].to_uppercase();
                sections
                    .entry(current_section.clone())
                    .or_insert_with(HashMap::new);
                continue;
            }

            // Parse key=value pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_uppercase();
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                if !current_section.is_empty() {
                    if let Some(section) = sections.get_mut(&current_section) {
                        section.insert(key, value);
                    }
                }
            }
        }

        // Get the PGGLOBAL section
        let pgglobal = sections.get("PGGLOBAL").ok_or_else(|| {
            CsvAnalyzerError::ConfigError("Missing [PGGLOBAL] section in config file".to_string())
        })?;

        // Extract required fields
        let host = pgglobal
            .get("HOSTNAME")
            .filter(|v| !v.is_empty())
            .ok_or_else(|| {
                CsvAnalyzerError::ConfigError(
                    "Missing or empty HOSTNAME in [PGGLOBAL] section".to_string(),
                )
            })?
            .clone();

        // Port is optional in the config, default to 5432
        let port = pgglobal
            .get("PORT")
            .and_then(|p| p.parse().ok())
            .unwrap_or(5432);

        let database = pgglobal
            .get("DATABASENAME")
            .filter(|v| !v.is_empty())
            .ok_or_else(|| {
                CsvAnalyzerError::ConfigError(
                    "Missing or empty DATABASENAME in [PGGLOBAL] section".to_string(),
                )
            })?
            .clone();

        let user = pgglobal
            .get("USERNAME")
            .filter(|v| !v.is_empty())
            .ok_or_else(|| {
                CsvAnalyzerError::ConfigError(
                    "Missing or empty USERNAME in [PGGLOBAL] section".to_string(),
                )
            })?
            .clone();

        let password = pgglobal
            .get("PASSWORD")
            .filter(|v| !v.is_empty())
            .ok_or_else(|| {
                CsvAnalyzerError::ConfigError(
                    "Missing or empty PASSWORD in [PGGLOBAL] section".to_string(),
                )
            })?
            .clone();

        Ok(DbConfig {
            host,
            port,
            database,
            user,
            password,
        })
    }

    /// Build PostgreSQL connection string
    pub fn connection_string(&self) -> String {
        format!(
            "host={} port={} dbname={} user={} password={}",
            self.host, self.port, self.database, self.user, self.password
        )
    }

    /// Build connection string for a specific user pool
    pub fn user_pool_connection_string(&self, ip_rw: &str, pool_name: &str) -> String {
        format!(
            "host={} port={} dbname={} user={} password={}",
            ip_rw, self.port, pool_name, self.user, self.password
        )
    }
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub akid: i64,
    pub locale: String,
    pub filename: String,
    pub db: DbConfig,
    pub scan_lines: usize,
    pub return_lines: usize,
}

impl Config {
    pub fn new(akid: i64, locale: String, filename: String) -> Result<Self> {
        Ok(Config {
            akid,
            locale,
            filename,
            db: DbConfig::from_env()?,
            scan_lines: crate::types::constants::MAX_SCAN_LINES,
            return_lines: crate::types::constants::MAX_RETURN_LINES,
        })
    }

    pub fn new_with_db(akid: i64, locale: String, filename: String, db: DbConfig) -> Self {
        Config {
            akid,
            locale,
            filename,
            db,
            scan_lines: crate::types::constants::MAX_SCAN_LINES,
            return_lines: crate::types::constants::MAX_RETURN_LINES,
        }
    }

    pub fn with_db_config(mut self, db: DbConfig) -> Self {
        self.db = db;
        self
    }
}
