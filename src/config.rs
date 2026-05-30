use std::fmt;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

#[derive(Clone, Default, Deserialize)]
pub struct AppConfig {
    /// Legacy SQLite path. Prefer `[backend].sqlite_path` for new configs.
    pub sqlite_path: Option<PathBuf>,
    #[serde(default)]
    pub backend: BackendConfig,
    #[serde(default)]
    pub server: ServerConfig,
}

#[derive(Clone, Default, Deserialize)]
pub struct BackendConfig {
    #[serde(default, alias = "type")]
    pub kind: BackendKind,
    pub sqlite_path: Option<PathBuf>,
    pub postgres_url: Option<String>,
    pub postgres_ssl_root_cert: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    #[default]
    Sqlite,
    Postgres,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ServerConfig {
    pub host: Option<IpAddr>,
    pub port: Option<u16>,
}

impl ServerConfig {
    pub fn resolve(&self) -> Result<SocketAddr> {
        let host = server_host_env()
            .transpose()?
            .or(self.host)
            .unwrap_or(IpAddr::from([127, 0, 0, 1]));
        let port = server_port_env()?.or(self.port).unwrap_or(0);
        Ok(SocketAddr::new(host, port))
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        if let Some(path) = config_path() {
            return Self::load_from_path(&path);
        }

        Ok(Self::default())
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file at {}", path.display()))?;
        let config: AppConfig = toml::from_str(&content)
            .with_context(|| format!("failed to parse config file at {}", path.display()))?;
        Ok(config)
    }

    pub fn resolve_sqlite_path(&self) -> Result<PathBuf> {
        if let Some(path) = sqlite_path_env()? {
            return Ok(path);
        }

        if let Some(path) = self.backend.sqlite_path.clone() {
            return Ok(path);
        }

        if let Some(path) = self.sqlite_path.clone() {
            return Ok(path);
        }

        default_sqlite_path()
    }

    pub fn resolve_postgres_url(&self) -> Result<String> {
        if let Some(url) = postgres_url_env()? {
            return Ok(url);
        }

        self.backend.postgres_url.clone().ok_or_else(|| {
            anyhow!("Postgres backend requires backend.postgres_url or TASKFORCE_POSTGRES_URL")
        })
    }

    pub fn resolve_postgres_ssl_root_cert(&self) -> Result<Option<PathBuf>> {
        if let Some(path) = postgres_ssl_root_cert_env()? {
            return Ok(Some(path));
        }

        Ok(self.backend.postgres_ssl_root_cert.clone())
    }
}

impl fmt::Debug for AppConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AppConfig")
            .field("sqlite_path", &self.sqlite_path)
            .field("backend", &self.backend)
            .field("server", &self.server)
            .finish()
    }
}

impl fmt::Debug for BackendConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let postgres_url = self.postgres_url.as_ref().map(|_| "<redacted>");
        formatter
            .debug_struct("BackendConfig")
            .field("kind", &self.kind)
            .field("sqlite_path", &self.sqlite_path)
            .field("postgres_url", &postgres_url)
            .field("postgres_ssl_root_cert", &self.postgres_ssl_root_cert)
            .finish()
    }
}

pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("config.toml"))
}

pub fn env_file_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("taskforce.env"))
}

fn config_dir() -> Option<PathBuf> {
    if let Some(xdg_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg_home).join("taskforce"));
    }

    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config").join("taskforce"))
}

pub fn sqlite_path_env() -> Result<Option<PathBuf>> {
    match std::env::var_os("TASKFORCE_SQLITE_PATH") {
        Some(path) => Ok(Some(PathBuf::from(path))),
        None => Ok(None),
    }
}

pub fn postgres_url_env() -> Result<Option<String>> {
    match std::env::var("TASKFORCE_POSTGRES_URL") {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(anyhow!("failed to read TASKFORCE_POSTGRES_URL: {err}")),
    }
}

pub fn postgres_ssl_root_cert_env() -> Result<Option<PathBuf>> {
    match std::env::var_os("TASKFORCE_POSTGRES_SSL_ROOT_CERT") {
        Some(path) => Ok(Some(PathBuf::from(path))),
        None => Ok(None),
    }
}

pub fn server_host_env() -> Option<Result<IpAddr>> {
    std::env::var("TASKFORCE_HOST").ok().map(|value| {
        IpAddr::from_str(&value).map_err(|_| anyhow!("invalid TASKFORCE_HOST: {value}"))
    })
}

pub fn server_port_env() -> Result<Option<u16>> {
    match std::env::var("TASKFORCE_PORT") {
        Ok(value) => value
            .parse::<u16>()
            .map(Some)
            .map_err(|_| anyhow!("invalid TASKFORCE_PORT: {value}")),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(anyhow!("failed to read TASKFORCE_PORT: {err}")),
    }
}

pub fn data_dir() -> Result<PathBuf> {
    if let Some(xdg_home) = std::env::var_os("XDG_DATA_HOME") {
        return Ok(PathBuf::from(xdg_home).join("taskforce"));
    }

    let home = std::env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("taskforce"))
}

fn default_sqlite_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("taskforce.db"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use std::net::{IpAddr, SocketAddr};

    use super::{AppConfig, BackendKind, ServerConfig};

    #[test]
    fn loads_sqlite_path_from_toml() -> Result<()> {
        let path = unique_temp_path("taskforce-backend-config");
        fs::write(&path, "sqlite_path = \"/tmp/taskforce.db\"\n")?;

        let config = AppConfig::load_from_path(&path)?;

        assert_eq!(config.sqlite_path, Some(PathBuf::from("/tmp/taskforce.db")));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn loads_backend_settings_from_toml() -> Result<()> {
        let path = unique_temp_path("taskforce-backend-config");
        fs::write(
            &path,
            "[backend]\nkind = \"sqlite\"\nsqlite_path = \"/tmp/backend.db\"\n",
        )?;

        let config = AppConfig::load_from_path(&path)?;

        assert_eq!(config.backend.kind, BackendKind::Sqlite);
        assert_eq!(
            config.backend.sqlite_path,
            Some(PathBuf::from("/tmp/backend.db"))
        );
        assert_eq!(
            config.resolve_sqlite_path()?,
            PathBuf::from("/tmp/backend.db")
        );
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn loads_postgres_backend_settings_from_toml() -> Result<()> {
        let path = unique_temp_path("taskforce-postgres-config");
        fs::write(
            &path,
            "[backend]\nkind = \"postgres\"\npostgres_url = \"postgresql://user:pass@db.example.com/postgres?sslmode=require\"\npostgres_ssl_root_cert = \"/tmp/supabase-prod-ca-2021.crt\"\n",
        )?;

        let config = AppConfig::load_from_path(&path)?;

        assert_eq!(config.backend.kind, BackendKind::Postgres);
        assert_eq!(
            config.backend.postgres_url.as_deref(),
            Some("postgresql://user:pass@db.example.com/postgres?sslmode=require")
        );
        assert_eq!(
            config.resolve_postgres_url()?,
            "postgresql://user:pass@db.example.com/postgres?sslmode=require"
        );
        assert_eq!(
            config.resolve_postgres_ssl_root_cert()?,
            Some(PathBuf::from("/tmp/supabase-prod-ca-2021.crt"))
        );
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn redacts_postgres_url_in_debug_output() {
        let config = AppConfig {
            sqlite_path: None,
            backend: super::BackendConfig {
                kind: BackendKind::Postgres,
                sqlite_path: None,
                postgres_url: Some("postgresql://postgres:secret@db.example.com/postgres".into()),
                postgres_ssl_root_cert: Some(PathBuf::from("/tmp/supabase-prod-ca-2021.crt")),
            },
            server: ServerConfig::default(),
        };

        let debug = format!("{config:?}");
        assert!(debug.contains("postgres_url: Some(\"<redacted>\")"));
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn loads_server_settings_from_toml() -> Result<()> {
        let path = unique_temp_path("taskforce-server-config");
        fs::write(&path, "[server]\nhost = \"0.0.0.0\"\nport = 9090\n")?;

        let config = AppConfig::load_from_path(&path)?;

        assert_eq!(config.server.host, Some(IpAddr::from([0, 0, 0, 0])));
        assert_eq!(config.server.port, Some(9090));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn resolves_server_addr_with_defaults() -> Result<()> {
        let resolved = ServerConfig::default().resolve()?;
        assert_eq!(resolved, SocketAddr::from(([127, 0, 0, 1], 0)));
        Ok(())
    }

    fn unique_temp_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}.toml"))
    }
}
