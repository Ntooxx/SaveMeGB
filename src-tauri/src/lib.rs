pub mod license;
pub mod manifest;
pub mod model;
pub mod progress;
pub mod registry;
pub mod safe_delete;
pub mod scanner;
pub mod settings;

use crate::model::{AppSettings, DeleteStrategy, PurgeReport, ScanReport, Whitelist};
use crate::progress::{sink_from_app, stdio_sink, ProgressSink};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri::Manager;

#[tauri::command]
async fn scan_system(
    app: AppHandle,
    mode: Option<String>,
    whitelist: Option<Whitelist>,
) -> Result<ScanReport, String> {
    let sink = sink_from_app(app);
    let mode = scanner::ScanMode::parse(mode.as_deref().unwrap_or("standard"));
    let wl = whitelist.unwrap_or_default();
    tauri::async_runtime::spawn_blocking(move || scanner::full_scan_with_mode(sink, mode, &wl))
        .await
        .map_err(|e| format!("scan task: {e}"))?
        .map_err(|e| format!("scan engine: {e}"))
}

#[tauri::command]
async fn purge_orphans(
    app: AppHandle,
    paths: Vec<String>,
    strategy: DeleteStrategy,
) -> Result<PurgeReport, String> {
    let pb: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    tauri::async_runtime::spawn_blocking(move || safe_delete::safe_purge(&pb, &strategy))
        .await
        .map_err(|e| format!("purge task: {e}"))?
        .map_err(|e| format!("purge engine: {e}"))
}

#[tauri::command]
fn manifest_age_days() -> Result<Option<u64>, String> {
    let store = manifest::ManifestStore::new().map_err(|e| e.to_string())?;
    store.age_days().map_err(|e| e.to_string())
}

#[tauri::command]
async fn refresh_manifest() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| {
        let store = manifest::ManifestStore::new().map_err(|e| e.to_string())?;
        store.download().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("refresh task: {e}"))?
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    settings::load(&app).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    settings::save(&app, &settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_in_explorer(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("path does not exist: {}", p.display()));
    }
    let target = if p.is_dir() {
        p
    } else {
        p.parent().map(|x| x.to_path_buf()).unwrap_or(p)
    };
    #[cfg(windows)]
    {
        let native = target.to_string_lossy().replace('/', "\\");
        std::process::Command::new("explorer.exe")
            .arg(&native)
            .spawn()
            .map_err(|e| format!("explorer spawn failed: {e}"))?;
    }
    #[cfg(not(windows))]
    {
        let _ = target;
    }
    Ok(())
}

#[tauri::command]
fn open_steam_app(appid: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(format!("steam://nav/games/details/{}", appid))
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn open_recycle_bin() -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer.exe")
            .arg("shell:RecycleBinFolder")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        let escaped = text.replace('"', "'");
        let script = format!("Set-Clipboard -Value \"{}\"", escaped);
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output();
    }
    Ok(())
}

#[tauri::command]
fn steam_cover_url(appid: String) -> String {
    format!("https://cdn.akamai.steamstatic.com/steam/apps/{}/library_600x900_2x.jpg", appid)
}

#[tauri::command]
fn export_report(app: AppHandle, format: String) -> Result<String, String> {
    let dir: PathBuf = app
        .path()
        .download_dir()
        .or_else(|_| app.path().home_dir())
        .map_err(|e| e.to_string())?;
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let path = match format.as_str() {
        "csv" => dir.join(format!("SaveMeGB-scan-{}.csv", stamp)),
        _ => dir.join(format!("SaveMeGB-scan-{}.json", stamp)),
    };
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn write_export(path: String, contents: String) -> Result<(), String> {
    std::fs::write(&path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
fn dir_size(path: String) -> Result<u64, String> {
    let p = PathBuf::from(path);
    if !p.exists() {
        return Ok(0);
    }
    let mut total = 0u64;
    for entry in walkdir::WalkDir::new(&p).into_iter().flatten() {
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    Ok(total)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub children: Vec<TreeNode>,
}

#[tauri::command]
fn dir_tree(path: String, max_depth: u32, max_entries: u32) -> Result<TreeNode, String> {
    let p = PathBuf::from(path);
    if !p.exists() {
        return Err("path does not exist".into());
    }
    let mut counter: u32 = 0;
    Ok(build_tree_node(&p, 0, max_depth, max_entries, &mut counter))
}

fn build_tree_node(
    path: &Path,
    depth: u32,
    max_depth: u32,
    max_entries: u32,
    counter: &mut u32,
) -> TreeNode {
    let meta = std::fs::metadata(path).ok();
    let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let name = path
        .file_name()
        .map(|s: &std::ffi::OsStr| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());
    let mut children = Vec::new();
    if is_dir && depth < max_depth && *counter < max_entries {
        if let Ok(rd) = std::fs::read_dir(path) {
            let mut entries: Vec<_> = rd.flatten().collect();
            entries.sort_by_key(|e| {
                std::fs::metadata(e.path())
                    .map(|m| std::cmp::Reverse(m.len()))
                    .unwrap_or(std::cmp::Reverse(0))
            });
            for entry in entries {
                if *counter >= max_entries {
                    break;
                }
                *counter += 1;
                children.push(build_tree_node(&entry.path(), depth + 1, max_depth, max_entries, counter));
            }
        }
    }
    TreeNode {
        name,
        path: path.to_string_lossy().to_string(),
        is_dir,
        size,
        children,
    }
}

#[tauri::command]
fn run_tests() -> Result<String, String> {
    let output = std::process::Command::new(env!("CARGO"))
        .args(["test", "--manifest-path", "src-tauri/Cargo.toml", "--tests"])
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::try_init();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let _ = settings::ensure(&handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_system,
            purge_orphans,
            manifest_age_days,
            refresh_manifest,
            get_settings,
            save_settings,
            open_in_explorer,
            open_steam_app,
            open_recycle_bin,
            steam_cover_url,
            export_report,
            write_export,
            dir_size,
            dir_tree,
            run_tests,
            copy_to_clipboard,
            license::activate_license,
            license::get_license,
            license::deactivate_license,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run_cli_scan_with(mode_str: &str) -> Result<ScanReport, String> {
    let sink: ProgressSink = stdio_sink();
    let mode = scanner::ScanMode::parse(mode_str);
    scanner::full_scan_with_mode(sink, mode, &Whitelist::default()).map_err(|e| e.to_string())
}

pub fn run_cli_scan() -> Result<ScanReport, String> {
    run_cli_scan_with("standard")
}

pub fn run_cli_refresh() -> Result<(), String> {
    let store = manifest::ManifestStore::new().map_err(|e| e.to_string())?;
    store.download().map_err(|e| e.to_string())
}

pub mod run_test {
    use std::path::Path;
    pub fn build_tree(p: &Path, max_depth: u32, max_entries: u32) -> crate::TreeNode {
        let mut counter = 0u32;
        super::build_tree_node(p, 0, max_depth, max_entries, &mut counter)
    }
}
