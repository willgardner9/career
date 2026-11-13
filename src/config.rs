use std::{env, net::SocketAddr};

use anyhow::{Context, Result};

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:3000";
const DEFAULT_DB_MAX_CONNECTIONS: u32 = 10;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub db_max_connections: u32,
}

impl Config {
    /// Read the configuration from the process environment.
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|key| env::var(key).ok())
    }

    /// Read the configuration from any key lookup. Tests use this
    /// function, so they do not change the process environment.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let database_url = lookup("DATABASE_URL").context("DATABASE_URL is not set")?;
        let bind_addr = lookup("BIND_ADDR")
            .unwrap_or_else(|| DEFAULT_BIND_ADDR.to_owned())
            .parse()
            .context("BIND_ADDR is not a valid socket address")?;
        let db_max_connections = match lookup("DB_MAX_CONNECTIONS") {
            Some(value) => value
                .parse()
                .context("DB_MAX_CONNECTIONS is not a valid number")?,
            None => DEFAULT_DB_MAX_CONNECTIONS,
        };

        Ok(Self {
            database_url,
            bind_addr,
            db_max_connections,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key| map.get(key).cloned()
    }

    #[test]
    fn uses_default_bind_addr() {
        let config = Config::from_lookup(lookup(&[("DATABASE_URL", "postgres://x")])).unwrap();
        assert_eq!(config.database_url, "postgres://x");
        assert_eq!(config.bind_addr, DEFAULT_BIND_ADDR.parse().unwrap());
        assert_eq!(config.db_max_connections, DEFAULT_DB_MAX_CONNECTIONS);
    }

    #[test]
    fn reads_db_max_connections() {
        let config = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://x"),
            ("DB_MAX_CONNECTIONS", "25"),
        ]))
        .unwrap();
        assert_eq!(config.db_max_connections, 25);
    }

    #[test]
    fn rejects_invalid_db_max_connections() {
        let result = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://x"),
            ("DB_MAX_CONNECTIONS", "many"),
        ]));
        assert!(result.is_err());
    }

    #[test]
    fn reads_bind_addr() {
        let config = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://x"),
            ("BIND_ADDR", "127.0.0.1:8080"),
        ]))
        .unwrap();
        assert_eq!(config.bind_addr, "127.0.0.1:8080".parse().unwrap());
    }

    #[test]
    fn requires_database_url() {
        assert!(Config::from_lookup(lookup(&[])).is_err());
    }

    #[test]
    fn rejects_invalid_bind_addr() {
        let result = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://x"),
            ("BIND_ADDR", "not-an-address"),
        ]));
        assert!(result.is_err());
    }
}
