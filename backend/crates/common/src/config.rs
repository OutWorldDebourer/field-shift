//! Configuration from `env.txt` (API keys, "label value" per line) and `infra/.env` (database).
//! Secret values are never logged: `Secrets` and `Config` have redacting Debug impls.
use std::{collections::HashMap, fmt, path::{Path, PathBuf}};

#[derive(Clone, Default)]
pub struct Secrets {
    pub nasa_api: Option<String>,
    pub earthdata: Option<String>,
    pub anthropic: Option<String>,
    pub gemini: Option<String>,
}

impl fmt::Debug for Secrets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = |o: &Option<String>| if o.is_some() { "set" } else { "missing" };
        write!(f, "Secrets {{ nasa_api: {}, earthdata: {}, anthropic: {}, gemini: {} }}",
            s(&self.nasa_api), s(&self.earthdata), s(&self.anthropic), s(&self.gemini))
    }
}

#[derive(Clone)]
pub struct Config {
    pub root: PathBuf,
    pub database_url_redacted: String,
    database_url: String,
    pub bind: String,
    pub secrets: Secrets,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Config {{ root: {:?}, db: {}, bind: {}, {:?} }}", self.root, self.database_url_redacted, self.bind, self.secrets)
    }
}

impl Config {
    pub fn database_url(&self) -> &str { &self.database_url }

    /// Loads from the project root (folder holding `BUILD_PLAN.md` and `infra/`), from FIELDSHIFT_ROOT or by walking up.
    pub fn load() -> anyhow::Result<Self> {
        let root = std::env::var("FIELDSHIFT_ROOT").map(PathBuf::from).ok()
            .or_else(|| find_root(&std::env::current_dir().ok()?))
            .ok_or_else(|| anyhow::anyhow!("project root not found (set FIELDSHIFT_ROOT)"))?;
        let secrets = read_secrets(&root.join("env.txt"));
        let db = read_dotenv(&root.join("infra/.env"));
        let get = |k: &str| std::env::var(k).ok().or_else(|| db.get(k).cloned()).unwrap_or_default();
        let (user, pass, name) = (get("DB_USER"), get("DB_PASSWORD"), get("DB_NAME"));
        let host = std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1:3306".into());
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| format!("mysql://{user}:{pass}@{host}/{name}"));
        Ok(Self {
            database_url_redacted: format!("mysql://{user}:***@{host}/{name}"),
            database_url,
            bind: std::env::var("FIELDSHIFT_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into()),
            secrets,
            root,
        })
    }
}

fn find_root(start: &Path) -> Option<PathBuf> {
    start.ancestors().find(|p| p.join("infra").is_dir() && p.join("BUILD_PLAN.md").exists()).map(Path::to_path_buf)
}

fn read_secrets(path: &Path) -> Secrets {
    let mut s = Secrets::default();
    let Ok(text) = std::fs::read_to_string(path) else { return s };
    for line in text.lines() {
        let mut it = line.split_whitespace();
        let (Some(k), Some(v)) = (it.next(), it.next()) else { continue };
        let v = Some(v.to_string());
        match k {
            "api.nasa.gov" => s.nasa_api = v,
            "earthdata" => s.earthdata = v,
            "anthropic" => s.anthropic = v,
            "gemini" => s.gemini = v,
            _ => {}
        }
    }
    s
}

fn read_dotenv(path: &Path) -> HashMap<String, String> {
    std::fs::read_to_string(path).unwrap_or_default().lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().into(), v.trim().trim_matches('"').into()))
        .collect()
}
