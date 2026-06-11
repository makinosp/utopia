use std::env;

use anyhow::{bail, Context};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub argon2_memory_cost: u32,
    pub argon2_time_cost: u32,
    pub argon2_parallelism: u32,
    pub token_cache_ttl_secs: u64,
    pub negative_token_cache_ttl_secs: u64,
    pub token_cache_max_capacity: u64,
    pub app_port: u16,
    pub log_level: String,
    pub bootstrap_key: String,
    pub bootstrap_user_email: String,
    pub strict_ssl: bool,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let strict_ssl = env::var("APP_STRICT_SSL")
            .ok()
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(true);

        let config = Self {
            database_url: required("DATABASE_URL")?,
            argon2_memory_cost: parse_required("ARGON2_MEMORY_COST")?,
            argon2_time_cost: parse_required("ARGON2_TIME_COST")?,
            argon2_parallelism: parse_required("ARGON2_PARALLELISM")?,
            token_cache_ttl_secs: parse_required("TOKEN_CACHE_TTL_SECS")?,
            negative_token_cache_ttl_secs: parse_required("NEGATIVE_TOKEN_CACHE_TTL_SECS")?,
            token_cache_max_capacity: parse_required("TOKEN_CACHE_MAX_CAPACITY")?,
            app_port: parse_required("APP_PORT")?,
            log_level: required("LOG_LEVEL")?,
            bootstrap_key: required("BOOTSTRAP_KEY")?,
            bootstrap_user_email: required("BOOTSTRAP_USER_EMAIL")?,
            strict_ssl,
        };

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.argon2_memory_cost < 65_536 {
            bail!(
                "ARGON2_MEMORY_COST must be >= 65536, got {}",
                self.argon2_memory_cost
            );
        }

        if self.argon2_time_cost < 3 {
            bail!(
                "ARGON2_TIME_COST must be >= 3, got {}",
                self.argon2_time_cost
            );
        }

        if self.argon2_parallelism < 1 {
            bail!(
                "ARGON2_PARALLELISM must be >= 1, got {}",
                self.argon2_parallelism
            );
        }

        if self.bootstrap_key.len() < 16 {
            bail!("BOOTSTRAP_KEY must be at least 16 characters");
        }

        if self.strict_ssl
            && !self.database_url.contains("sslmode=require")
            && !self.database_url.contains("ssl_mode=require")
        {
            bail!("DATABASE_URL must enforce TLS with sslmode=require when APP_STRICT_SSL=true");
        }

        Ok(())
    }
}

fn required(name: &str) -> anyhow::Result<String> {
    env::var(name).with_context(|| format!("missing required env var {name}"))
}

fn parse_required<T>(name: &str) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let raw = required(name)?;
    raw.parse::<T>()
        .map_err(|err| anyhow::anyhow!("failed to parse {name}: {err}"))
}
