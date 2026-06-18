use crate::model::ManifestEntry;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const MANIFEST_URL: &str = "https://raw.githubusercontent.com/mtkennerly/ludusavi-manifest/master/data/manifest.json";
const MAX_AGE_DAYS: u64 = 7;

pub struct ManifestStore {
    pub path: PathBuf,
}

impl ManifestStore {
    pub fn new() -> Result<Self> {
        let dir = directories::ProjectDirs::from("com", "savemegb", "SaveMeGB")
            .context("could not resolve project dirs")?
            .data_dir()
            .to_path_buf();
        std::fs::create_dir_all(&dir).context("create data dir")?;
        Ok(Self {
            path: dir.join("manifest.json"),
        })
    }

    pub fn age_days(&self) -> Result<Option<u64>> {
        if !self.path.exists() {
            return Ok(None);
        }
        let meta = std::fs::metadata(&self.path)?;
        let modified = meta.modified()?;
        let elapsed = SystemTime::now()
            .duration_since(modified)
            .unwrap_or_default();
        Ok(Some(elapsed.as_secs() / 86_400))
    }

    pub fn ensure_fresh(&self) -> Result<()> {
        let fresh = self.age_days()?.map(|d| d < MAX_AGE_DAYS).unwrap_or(false);
        if fresh {
            return Ok(());
        }
        self.download()
    }

    pub async fn download_async(&self) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .user_agent("SaveMeGB/0.1")
            .build()?;
        let body = client
            .get(MANIFEST_URL)
            .send()
            .await
            .context("download request")?
            .error_for_status()
            .context("download status")?
            .bytes()
            .await
            .context("download body")?;
        let tmp = self.path.with_extension("json.tmp");
        tokio::fs::write(&tmp, &body)
            .await
            .context("write temp manifest")?;
        tokio::fs::rename(&tmp, &self.path)
            .await
            .context("rename manifest")?;
        Ok(())
    }

    pub fn download(&self) -> Result<()> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("build tokio runtime")?;
        rt.block_on(self.download_async())
    }

    pub fn load(&self) -> Result<Vec<ManifestEntry>> {
        let raw = std::fs::read_to_string(&self.path).context("read manifest")?;
        let v: Value = serde_json::from_str(&raw).context("parse manifest")?;
        let games = v
            .get("games")
            .and_then(|g| g.as_object())
            .context("manifest missing `games` object")?;
        let mut out = Vec::with_capacity(games.len());
        for (name, body) in games {
            out.push(parse_entry(name, body));
        }
        Ok(out)
    }

    pub fn try_load_cached(&self) -> Result<Vec<ManifestEntry>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        self.load()
    }
}

fn parse_entry(name: &str, body: &Value) -> ManifestEntry {
    let steam_app_id = body
        .get("steam")
        .and_then(|s| s.as_object())
        .and_then(|s| s.get("id"))
        .and_then(|i| i.as_str())
        .map(String::from);
    let mut save_paths = Vec::new();
    if let Some(files) = body.get("files").and_then(|f| f.as_object()) {
        for (_key, val) in files {
            if let Some(arr) = val.as_array() {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        save_paths.push(s.to_string());
                    }
                }
            }
        }
    }
    let registry_keys = body
        .get("registry")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    ManifestEntry {
        name: name.to_string(),
        steam_app_id,
        save_paths,
        registry_keys,
    }
}
