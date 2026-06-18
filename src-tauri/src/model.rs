use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum GameSource {
    Steam,
    Epic,
    Xbox,
    GOG,
    Manual,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledGame {
    pub id: String,
    pub name: String,
    pub source: GameSource,
    pub install_path: PathBuf,
    pub app_id: Option<String>,
    pub size_bytes: u64,
    pub estimated_save_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManifestEntry {
    pub name: String,
    pub steam_app_id: Option<String>,
    pub save_paths: Vec<String>,
    pub registry_keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum FileCategory {
    Saves,
    Cache,
    Shaders,
    Logs,
    Crashes,
    Settings,
    Backups,
    Other,
}

impl FileCategory {
    pub fn label(&self) -> &'static str {
        match self {
            FileCategory::Saves => "Saves",
            FileCategory::Cache => "Cache",
            FileCategory::Shaders => "Shaders",
            FileCategory::Logs => "Logs",
            FileCategory::Crashes => "Crash dumps",
            FileCategory::Settings => "Settings",
            FileCategory::Backups => "Backups",
            FileCategory::Other => "Other",
        }
    }

    pub fn is_safer_to_delete(&self) -> bool {
        matches!(
            self,
            FileCategory::Cache | FileCategory::Shaders | FileCategory::Crashes | FileCategory::Logs
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanedFile {
    pub id: String,
    pub game_hint: String,
    pub category: FileCategory,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub file_count: u32,
    pub reason: String,
    pub last_modified: Option<String>,
    pub confidence: u8,
    pub source: Option<GameSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeleteStrategy {
    RecycleBin,
    BackupFolder { path: PathBuf },
    DirectDelete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub id: String,
    pub installed_games: Vec<InstalledGame>,
    pub orphaned_files: Vec<OrphanedFile>,
    pub total_reclaimable_bytes: u64,
    pub safe_to_delete_bytes: u64,
    pub scanned_at: String,
    pub duration_ms: u64,
    pub manifest_age_days: Option<u64>,
    pub stages: Vec<StageResult>,
    pub system: SystemInfo,
    pub category_breakdown: Vec<CategoryStat>,
    pub publisher_breakdown: Vec<PublisherStat>,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStat {
    pub category: String,
    pub count: u32,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherStat {
    pub publisher: String,
    pub count: u32,
    pub bytes: u64,
    pub is_game_studio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Whitelist {
    pub paths: Vec<String>,
    pub publishers: Vec<String>,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub orphaned_files: u32,
    pub total_reclaimable_bytes: u64,
    pub safe_to_delete_bytes: u64,
    pub games: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    pub name: String,
    pub duration_ms: u64,
    pub items: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemInfo {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub hostname: String,
    pub windows_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub stage: String,
    pub current: u32,
    pub total: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeReport {
    pub job_id: String,
    pub moved: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
    pub errors: Vec<PurgeError>,
    pub bytes_freed: u64,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeError {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub backup_folder: Option<PathBuf>,
    pub auto_refresh_manifest: bool,
    pub auto_scan_on_launch: bool,
    pub smart_clean_categories: Vec<FileCategory>,
    pub notifications_enabled: bool,
    pub min_confidence: u8,
    pub scan_mode: String,
    pub whitelist: Whitelist,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            backup_folder: None,
            auto_refresh_manifest: true,
            auto_scan_on_launch: false,
            smart_clean_categories: vec![
                FileCategory::Cache,
                FileCategory::Shaders,
                FileCategory::Crashes,
            ],
            notifications_enabled: true,
            min_confidence: 50,
            scan_mode: "standard".into(),
            whitelist: Whitelist::default(),
        }
    }
}
