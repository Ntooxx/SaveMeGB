use crate::model::AppSettings;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const SETTINGS_FILE: &str = "settings.json";

fn settings_path(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_config_dir()
        .context("resolve app config dir")?;
    std::fs::create_dir_all(&dir).context("create config dir")?;
    Ok(dir.join(SETTINGS_FILE))
}

pub fn ensure(app: &AppHandle) -> Result<AppSettings> {
    let path = settings_path(app)?;
    if !path.exists() {
        let defaults = AppSettings::default();
        let body = serde_json::to_string_pretty(&defaults)?;
        std::fs::write(&path, body)?;
        return Ok(defaults);
    }
    let body = std::fs::read_to_string(&path)?;
    let parsed: AppSettings = serde_json::from_str(&body).unwrap_or_default();
    Ok(parsed)
}

pub fn load(app: &AppHandle) -> Result<AppSettings> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let body = std::fs::read_to_string(&path)?;
    let parsed: AppSettings = serde_json::from_str(&body).unwrap_or_default();
    Ok(parsed)
}

pub fn save(app: &AppHandle, s: &AppSettings) -> Result<()> {
    let path = settings_path(app)?;
    let body = serde_json::to_string_pretty(s)?;
    std::fs::write(&path, body)?;
    Ok(())
}
