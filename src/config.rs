use std::{env, net::SocketAddr};

use anyhow::{Context, Result};
use axum::http::HeaderValue;

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:3000";
const DEFAULT_DB_MAX_CONNECTIONS: u32 = 10;
const DEFAULT_CORS_ALLOWED_ORIGINS: &str = "https://app.dearest.rsvp";

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub db_max_connections: u32,
    /// Browser origins that may call the API. Each origin has a scheme and a host.
    pub cors_allowed_origins: Vec<HeaderValue>,
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

        let cors_allowed_origins = lookup("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|| DEFAULT_CORS_ALLOWED_ORIGINS.to_owned())
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .map(|origin| {
                origin.parse().with_context(|| {
                    format!("CORS_ALLOWED_ORIGINS has an invalid origin: {origin}")
                })
            })
            .collect::<Result<Vec<HeaderValue>>>()?;

        Ok(Self {
            database_url,
            bind_addr,
            db_max_connections,
            cors_allowed_origins,
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
        assert_eq!(
            config.cors_allowed_origins,
            vec![HeaderValue::from_static("https://app.dearest.rsvp")]
        );
    }

    #[test]
    fn reads_cors_allowed_origins() {
        let config = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://x"),
            (
                "CORS_ALLOWED_ORIGINS",
                "http://localhost:5173, https://app.dearest.rsvp",
            ),
        ]))
        .unwrap();
        assert_eq!(
            config.cors_allowed_origins,
            vec![
                HeaderValue::from_static("http://localhost:5173"),
                HeaderValue::from_static("https://app.dearest.rsvp"),
            ]
        );
    }

    #[test]
    fn rejects_invalid_cors_allowed_origins() {
        let result = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://x"),
            ("CORS_ALLOWED_ORIGINS", "bad\nvalue"),
        ]));
        assert!(result.is_err());
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
