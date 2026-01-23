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

impl Default for DbConfig {
    fn default() -> Self {
        DbConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "mailjet".to_string(),
            user: "postgres".to_string(),
            password: String::new(),
        }
    }
}

impl DbConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        DbConfig {
            host: env::var("PGHOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("PGPORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
            database: env::var("PGDATABASE").unwrap_or_else(|_| "mailjet".to_string()),
            user: env::var("PGUSER").unwrap_or_else(|_| "postgres".to_string()),
            password: env::var("PGPASSWORD").unwrap_or_default(),
        }
    }

    /// Load configuration from a config file (mailjet.conf format)
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            CsvAnalyzerError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        let mut config = DbConfig::default();
        let mut values: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_lowercase();
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                values.insert(key, value);
            }
        }

        if let Some(host) = values.get("db_host").or_else(|| values.get("pghost")) {
            config.host = host.clone();
        }
        if let Some(port) = values.get("db_port").or_else(|| values.get("pgport")) {
            if let Ok(p) = port.parse() {
                config.port = p;
            }
        }
        if let Some(db) = values.get("db_name").or_else(|| values.get("pgdatabase")) {
            config.database = db.clone();
        }
        if let Some(user) = values.get("db_user").or_else(|| values.get("pguser")) {
            config.user = user.clone();
        }
        if let Some(pass) = values
            .get("db_password")
            .or_else(|| values.get("pgpassword"))
        {
            config.password = pass.clone();
        }

        Ok(config)
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
    pub fn new(akid: i64, locale: String, filename: String) -> Self {
        Config {
            akid,
            locale,
            filename,
            db: DbConfig::from_env(),
            scan_lines: crate::types::constants::MAX_SCAN_LINES,
            return_lines: crate::types::constants::MAX_RETURN_LINES,
        }
    }

    pub fn with_db_config(mut self, db: DbConfig) -> Self {
        self.db = db;
        self
    }
}
