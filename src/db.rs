use crate::config::DbConfig;
use crate::error::{CsvAnalyzerError, Result};
use crate::types::{ContactProperty, DataType};
use postgres::{Client, NoTls};

/// Database connection manager
pub struct DbConnection {
    global_client: Option<Client>,
    user_client: Option<Client>,
    config: DbConfig,
}

/// Pool info from app table
#[derive(Debug)]
pub struct PoolInfo {
    pub pool: i32,
    pub ip_rw: String,
    pub db_version: i32,
}

impl DbConnection {
    pub fn new(config: DbConfig) -> Self {
        DbConnection {
            global_client: None,
            user_client: None,
            config,
        }
    }

    /// Connect to global database
    pub fn connect_global(&mut self) -> Result<()> {
        let conn_str = self.config.connection_string();
        let client = Client::connect(&conn_str, NoTls).map_err(|e| {
            CsvAnalyzerError::DatabaseError(format!("Failed to connect to global database: {}", e))
        })?;
        self.global_client = Some(client);
        Ok(())
    }

    /// Query app table for pool info
    pub fn get_pool_info(&mut self, akid: i64) -> Result<PoolInfo> {
        let client = self.global_client.as_mut().ok_or_else(|| {
            CsvAnalyzerError::DatabaseError("Global connection not established".to_string())
        })?;

        let sql = format!(
            "SELECT pool, ip_rw, db_version FROM app WHERE id = {}",
            akid
        );

        // Use simple_query to avoid prepared statement issues with PgBouncer
        let messages = client.simple_query(&sql).map_err(|e| {
            CsvAnalyzerError::DatabaseError(format!("Failed to query app table: {}", e))
        })?;

        // Find the first Row message and extract data
        for message in messages {
            if let postgres::SimpleQueryMessage::Row(row) = message {
                // row.get() returns Option<&str>
                let pool_str = row.get(0).ok_or_else(|| {
                    CsvAnalyzerError::DatabaseError("Column 0 (pool) is NULL".to_string())
                })?;
                let pool: i32 = pool_str.parse().map_err(|_| {
                    CsvAnalyzerError::DatabaseError(format!(
                        "Failed to parse pool value: '{}'",
                        pool_str
                    ))
                })?;

                let ip_rw = row
                    .get(1)
                    .ok_or_else(|| {
                        CsvAnalyzerError::DatabaseError("Column 1 (ip_rw) is NULL".to_string())
                    })?
                    .to_string();

                let db_version_str = row.get(2).ok_or_else(|| {
                    CsvAnalyzerError::DatabaseError("Column 2 (db_version) is NULL".to_string())
                })?;
                let db_version: i32 = db_version_str.parse().map_err(|_| {
                    CsvAnalyzerError::DatabaseError(format!(
                        "Failed to parse db_version value: '{}'",
                        db_version_str
                    ))
                })?;

                return Ok(PoolInfo {
                    pool,
                    ip_rw,
                    db_version,
                });
            }
        }

        Err(CsvAnalyzerError::DatabaseError(
            "No rows returned from app table query".to_string(),
        ))
    }

    /// Connect to user pool database
    pub fn connect_user_pool(&mut self, pool_info: &PoolInfo) -> Result<()> {
        // Format the pool number as a database name with 7-digit zero padding
        let pool_name = format!("p{:07}", pool_info.pool);
        let conn_str = self
            .config
            .user_pool_connection_string(&pool_info.ip_rw, &pool_name);

        let client = Client::connect(&conn_str, NoTls).map_err(|e| {
            CsvAnalyzerError::DatabaseError(format!("Failed to connect to user pool: {}", e))
        })?;
        self.user_client = Some(client);
        Ok(())
    }

    /// Query contact metadata for an account
    pub fn get_contact_properties(&mut self, _akid: i64) -> Result<Vec<ContactProperty>> {
        let client = self.user_client.as_mut().ok_or_else(|| {
            CsvAnalyzerError::DatabaseError("User pool connection not established".to_string())
        })?;

        // mnStatic = 0 (static namespace for contact properties)
        // let sql = format!(
        //     "SELECT name, datatype FROM t{_akid}_contact_meta WHERE namespace = 0",
        // );
        let sql = "SELECT name, datatype FROM contact_meta WHERE namespace = 0".to_string();

        // Use simple_query to avoid prepared statement issues with PgBouncer
        let messages = client.simple_query(&sql).map_err(|e| {
            CsvAnalyzerError::DatabaseError(format!("Failed to query contact_meta: {}", e))
        })?;

        let mut properties = Vec::new();
        let mut row_count = 0;

        // Iterate through messages and process Row variants
        for message in messages {
            if let postgres::SimpleQueryMessage::Row(row) = message {
                // row.get() returns Option<&str>
                let name = row
                    .get(0)
                    .ok_or_else(|| {
                        CsvAnalyzerError::DatabaseError(format!(
                            "Column 0 (name) is NULL at row {}",
                            row_count
                        ))
                    })?
                    .to_string();

                let datatype_str = row.get(1).ok_or_else(|| {
                    CsvAnalyzerError::DatabaseError(format!(
                        "Column 1 (datatype) is NULL at row {}",
                        row_count
                    ))
                })?;

                let datatype_int: i32 = datatype_str.parse().map_err(|_| {
                    CsvAnalyzerError::DatabaseError(format!(
                        "Failed to parse datatype value '{}' at row {}",
                        datatype_str, row_count
                    ))
                })?;

                let datatype = match datatype_int {
                    0 => DataType::String,
                    1 => DataType::Integer,
                    2 => DataType::Float,
                    3 => DataType::Boolean,
                    4 => DataType::DateTime,
                    _ => DataType::String,
                };

                #[cfg(debug_assertions)]
                eprintln!(
                    "DEBUG: Parsed property {} - name: {}, datatype: {:?}",
                    row_count, name, datatype
                );
                properties.push(ContactProperty { name, datatype });
                row_count += 1;
            }
        }

        Ok(properties)
    }

    /// Disconnect from all databases
    pub fn disconnect(&mut self) {
        self.global_client = None;
        self.user_client = None;
    }
}

impl Drop for DbConnection {
    fn drop(&mut self) {
        self.disconnect();
    }
}

/// Match a header name against known contact properties
pub fn match_property<'a>(
    header: &str,
    properties: &'a [ContactProperty],
) -> Option<&'a ContactProperty> {
    let header_lower = header.to_lowercase();
    properties
        .iter()
        .find(|p| p.name.to_lowercase() == header_lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_property() {
        let properties = vec![
            ContactProperty {
                name: "email".to_string(),
                datatype: DataType::String,
            },
            ContactProperty {
                name: "FirstName".to_string(),
                datatype: DataType::String,
            },
            ContactProperty {
                name: "Age".to_string(),
                datatype: DataType::Integer,
            },
        ];

        // Exact match (case-insensitive)
        let matched = match_property("firstname", &properties);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().name, "FirstName");

        // No match
        let not_matched = match_property("unknown", &properties);
        assert!(not_matched.is_none());
    }
}
