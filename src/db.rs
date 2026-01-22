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

        let row = client.query_one(&sql, &[]).map_err(|e| {
            CsvAnalyzerError::DatabaseError(format!("Failed to query app table: {}", e))
        })?;

        Ok(PoolInfo {
            pool: row.get(0),
            ip_rw: row.get(1),
            db_version: row.get(2),
        })
    }

    /// Connect to user pool database
    pub fn connect_user_pool(&mut self, pool_info: &PoolInfo) -> Result<()> {
        let pool_name = format!("pool{:02}", pool_info.pool);
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
    pub fn get_contact_properties(&mut self, akid: i64) -> Result<Vec<ContactProperty>> {
        let client = self.user_client.as_mut().ok_or_else(|| {
            CsvAnalyzerError::DatabaseError("User pool connection not established".to_string())
        })?;

        // mnStatic = 0 (static namespace for contact properties)
        let sql = format!(
            "SELECT name, datatype FROM t{}_contact_meta WHERE namespace = 0",
            akid
        );

        let rows = client.query(&sql, &[]).map_err(|e| {
            CsvAnalyzerError::DatabaseError(format!("Failed to query contact_meta: {}", e))
        })?;

        let mut properties = Vec::new();
        for row in rows {
            let name: String = row.get(0);
            let datatype_int: i32 = row.get(1);

            let datatype = match datatype_int {
                0 => DataType::String,
                1 => DataType::Integer,
                2 => DataType::Float,
                3 => DataType::Boolean,
                4 => DataType::DateTime,
                _ => DataType::String,
            };

            properties.push(ContactProperty { name, datatype });
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
