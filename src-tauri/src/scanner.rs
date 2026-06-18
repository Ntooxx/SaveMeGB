use crate::manifest::ManifestStore;
use crate::model::{
    CategoryStat, FileCategory, GameSource, InstalledGame, OrphanedFile, PublisherStat,
    ScanProgress, ScanReport, StageResult, SystemInfo, Whitelist,
};
use crate::progress::ProgressSink;
use crate::registry;
use anyhow::Result;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanMode {
    Quick,
    Standard,
    Deep,
}

impl ScanMode {
    pub fn parse(s: &str) -> Self {
        match s {
            "quick" => ScanMode::Quick,
            "deep" => ScanMode::Deep,
            _ => ScanMode::Standard,
        }
    }
}

pub fn full_scan_with_mode(
    sink: ProgressSink,
    mode: ScanMode,
    whitelist: &Whitelist,
) -> Result<ScanReport> {
    let started = Instant::now();
    let mut stages: Vec<StageResult> = Vec::new();

    emit(&sink, "init", 0, 1, "Preparing scan");
    let scan_id = uuid::Uuid::new_v4().to_string();
    let installed = registry::scan_all().unwrap_or_default();
    emit(
        &sink,
        "launchers",
        installed.len() as u32,
        installed.len() as u32,
        &format!("Found {} installed game(s)", installed.len()),
    );
    stages.push(StageResult {
        name: "launchers".into(),
        duration_ms: started.elapsed().as_millis() as u64,
        items: installed.len() as u32,
    });

    let stage_start = Instant::now();
    let store = ManifestStore::new().ok();
    let manifest = match &store {
        Some(s) => {
            emit(&sink, "manifest", 0, 1, "Refreshing game manifest");
            let _ = s.ensure_fresh();
            s.try_load_cached().unwrap_or_default()
        }
        None => Vec::new(),
    };
    emit(
        &sink,
        "manifest",
        manifest.len() as u32,
        manifest.len() as u32,
        &format!("Loaded {} manifest entries", manifest.len()),
    );
    stages.push(StageResult {
        name: "manifest".into(),
        duration_ms: stage_start.elapsed().as_millis() as u64,
        items: manifest.len() as u32,
    });

    let stage_start = Instant::now();
    emit(&sink, "scan", 0, 1, "Scanning user data");
    let mut orphans = find_orphans(&installed, &manifest, mode, whitelist, &sink);
    orphans.retain(|o| !is_whitelisted(o, whitelist));
    orphans.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    let total: u64 = orphans.iter().map(|o| o.size_bytes).sum();
    let safe_total: u64 = orphans
        .iter()
        .filter(|o| o.category.is_safer_to_delete())
        .map(|o| o.size_bytes)
        .sum();
    let (cat, pub_) = aggregate(&orphans);
    emit(
        &sink,
        "scan",
        orphans.len() as u32,
        orphans.len() as u32,
        &format!("Identified {} candidate folder(s)", orphans.len()),
    );
    stages.push(StageResult {
        name: "scan".into(),
        duration_ms: stage_start.elapsed().as_millis() as u64,
        items: orphans.len() as u32,
    });

    let manifest_age = store.as_ref().and_then(|s| s.age_days().ok().flatten());

    let report = ScanReport {
        id: scan_id,
        installed_games: installed,
        orphaned_files: orphans,
        total_reclaimable_bytes: total,
        safe_to_delete_bytes: safe_total,
        scanned_at: Utc::now().to_rfc3339(),
        duration_ms: started.elapsed().as_millis() as u64,
        manifest_age_days: manifest_age,
        stages,
        system: collect_system_info(),
        category_breakdown: cat,
        publisher_breakdown: pub_,
        mode: match mode {
            ScanMode::Quick => "quick".into(),
            ScanMode::Standard => "standard".into(),
            ScanMode::Deep => "deep".into(),
        },
    };
    emit(&sink, "done", 1, 1, "Scan complete");
    Ok(report)
}

pub fn full_scan(sink: ProgressSink) -> Result<ScanReport> {
    full_scan_with_mode(sink, ScanMode::Standard, &Whitelist::default())
}

fn is_whitelisted(o: &OrphanedFile, wl: &Whitelist) -> bool {
    let p = o.path.to_string_lossy().to_lowercase();
    for w in &wl.paths {
        if p.contains(&w.to_lowercase()) {
            return true;
        }
    }
    for w in &wl.names {
        if o.game_hint.to_lowercase() == w.to_lowercase() {
            return true;
        }
    }
    for w in &wl.publishers {
        if o.path
            .components()
            .any(|c| c.as_os_str().to_string_lossy().to_lowercase() == w.to_lowercase())
        {
            return true;
        }
    }
    false
}

fn aggregate(orphans: &[OrphanedFile]) -> (Vec<CategoryStat>, Vec<PublisherStat>) {
    let mut by_cat: HashMap<String, (u32, u64)> = HashMap::new();
    let mut by_pub: HashMap<String, (u32, u64, bool)> = HashMap::new();
    for o in orphans {
        let entry = by_cat.entry(o.category.label().to_string()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += o.size_bytes;
        if let Some(src) = o.source {
            let pub_name = format!("{:?}", src);
            let e = by_pub.entry(pub_name).or_insert((0, 0, false));
            e.0 += 1;
            e.1 += o.size_bytes;
            e.2 = true;
        } else {
            let publisher = o
                .path
                .components()
                .rev()
                .nth(1)
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".into());
            let is_studio = is_likely_publisher(&publisher.to_lowercase());
            let e = by_pub.entry(publisher).or_insert((0, 0, false));
            e.0 += 1;
            e.1 += o.size_bytes;
            e.2 = is_studio;
        }
    }
    let mut cat: Vec<CategoryStat> = by_cat
        .into_iter()
        .map(|(k, v)| CategoryStat {
            category: k,
            count: v.0,
            bytes: v.1,
        })
        .collect();
    cat.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let mut pub_: Vec<PublisherStat> = by_pub
        .into_iter()
        .map(|(k, v)| PublisherStat {
            publisher: k,
            count: v.0,
            bytes: v.1,
            is_game_studio: v.2,
        })
        .collect();
    pub_.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    (cat, pub_)
}

fn emit(sink: &ProgressSink, stage: &str, current: u32, total: u32, message: &str) {
    sink(ScanProgress {
        stage: stage.to_string(),
        current,
        total,
        message: message.to_string(),
    });
}

fn find_orphans(
    installed: &[InstalledGame],
    manifest: &[crate::model::ManifestEntry],
    mode: ScanMode,
    whitelist: &Whitelist,
    sink: &ProgressSink,
) -> Vec<OrphanedFile> {
    let mut results = Vec::new();
    let installed_names: HashSet<String> = installed
        .iter()
        .map(|g| g.name.to_lowercase())
        .collect();
    let known_steamapps: HashSet<PathBuf> = installed
        .iter()
        .map(|g| g.install_path.clone())
        .collect();
    let mut roots = user_save_roots();
    if mode == ScanMode::Deep {
        if let Some(home) = directories::UserDirs::new().map(|u| u.home_dir().to_path_buf()) {
            for extra in [
                home.join("Documents"),
                home.join("Downloads"),
            ] {
                if extra.exists() {
                    roots.push(extra);
                }
            }
        }
    }
    let total_roots = roots.len() as u32;

    for (i, root) in roots.iter().enumerate() {
        if !root.exists() {
            continue;
        }
        emit(
            sink,
            "scan",
            i as u32,
            total_roots,
            &format!("Walking {}", root.display()),
        );
        let entries = match std::fs::read_dir(root) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for publisher_entry in entries.flatten() {
            if !publisher_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let publisher = publisher_entry.file_name().to_string_lossy().to_string();
            let lp = publisher.to_lowercase();
            if is_denied_publisher(&lp) {
                continue;
            }
            if !is_likely_publisher(&lp) {
                continue;
            }
            let pub_path = publisher_entry.path();
            let pub_lower = lp.clone();
            let sub_entries = match std::fs::read_dir(&pub_path) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for game_entry in sub_entries.flatten() {
                if !game_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let game_name = game_entry.file_name().to_string_lossy().to_string();
                let lname = game_name.to_lowercase();
                if installed_names.contains(&lname) {
                    continue;
                }
                if is_denied_game_name(&lname) {
                    continue;
                }
                if whitelist
                    .names
                    .iter()
                    .any(|w| w.to_lowercase() == lname)
                {
                    continue;
                }
                let path = game_entry.path();
                if is_whitelisted_path(&path, whitelist) {
                    continue;
                }
                if path_under_any(&path, &known_steamapps) {
                    continue;
                }
                let Some(info) = classify_orphan(
                    &path,
                    &game_name,
                    &pub_lower,
                    manifest,
                    installed,
                ) else {
                    continue;
                };
                let mut info = info;
                let (size, count) = dir_size_and_count(&path);
                if size == 0 && count == 0 {
                    continue;
                }
                info.size_bytes = size;
                info.file_count = count;
                results.push(info);
            }
        }
    }
    if mode == ScanMode::Deep {
        deep_cache_scan(&mut results, sink);
    }
    results
}

fn is_whitelisted_path(path: &Path, wl: &Whitelist) -> bool {
    let p = path.to_string_lossy().to_lowercase();
    wl.paths.iter().any(|w| p.contains(&w.to_lowercase()))
}

fn deep_cache_scan(results: &mut Vec<OrphanedFile>, sink: &ProgressSink) {
    let Some(home) = directories::UserDirs::new().map(|u| u.home_dir().to_path_buf()) else {
        return;
    };
    let local = home.join("AppData").join("Local");
    let local_low = home.join("AppData").join("LocalLow");
    let candidates: Vec<(PathBuf, FileCategory, &'static str)> = vec![
        (local.join("NVIDIA").join("DXCache"), FileCategory::Shaders, "DirectX shader cache"),
        (local.join("NVIDIA").join("GLCache"), FileCategory::Shaders, "OpenGL shader cache"),
        (local.join("NVIDIA").join("ComputeCache"), FileCategory::Shaders, "NVIDIA compute cache"),
        (local.join("AMD").join("DxCache"), FileCategory::Shaders, "AMD DirectX cache"),
        (local.join("D3DSCache"), FileCategory::Shaders, "Direct3D shader cache"),
        (local.join("Intel").join("ShaderCache"), FileCategory::Shaders, "Intel shader cache"),
        (local.join("UnrealEngine").join("Common").join("ShaderBytecode"), FileCategory::Shaders, "Unreal shader bytecode"),
        (local.join("Unity").join("cache"), FileCategory::Cache, "Unity editor cache"),
        (local.join("Crashpad"), FileCategory::Crashes, "Crashpad reports"),
        (local_low.join("NVIDIA"), FileCategory::Shaders, "NVIDIA cache (LocalLow)"),
    ];
    emit(sink, "scan", 1, 2, "Deep cache scan");
    for (c, category, reason) in candidates {
        if !c.exists() {
            continue;
        }
        let (size, count) = dir_size_and_count(&c);
        if size == 0 {
            continue;
        }
        let already = results.iter().any(|o| o.path == c);
        if already {
            continue;
        }
        results.push(OrphanedFile {
            id: uuid::Uuid::new_v4().to_string(),
            game_hint: c
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Cache".into()),
            category,
            path: c.clone(),
            size_bytes: size,
            file_count: count,
            reason: reason.into(),
            last_modified: None,
            confidence: 85,
            source: None,
        });
    }
}

fn user_save_roots() -> Vec<PathBuf> {
    let Some(home) = directories::UserDirs::new().map(|u| u.home_dir().to_path_buf()) else {
        return vec![];
    };
    vec![
        home.join("AppData").join("Local"),
        home.join("AppData").join("LocalLow"),
        home.join("AppData").join("Roaming"),
        home.join("Documents").join("My Games"),
        home.join("Saved Games"),
    ]
}

fn path_under_any(path: &Path, roots: &HashSet<PathBuf>) -> bool {
    let mut cur = path.to_path_buf();
    loop {
        if roots.contains(&cur) {
            return true;
        }
        match cur.parent() {
            Some(p) => cur = p.to_path_buf(),
            None => return false,
        }
    }
}

fn is_denied_publisher(lp: &str) -> bool {
    const DENIED: &[&str] = &[
        "microsoft", "windows", "packages", "mozilla", "firefox", "thunderbird",
        "google", "chrome", "googlechrome", "bravesoftware", "brave-browser",
        "discord", "slack", "telegram", "signal", "whatsapp",
        "jetbrains", "intellijidea", "pycharm", "webstorm", "clion", "rider",
        "vscode", "code", "sublime text", "notepad++",
        "node-gyp", "npm", "yarn", "pnpm",
        "pip", "uv", "conda", "anaconda", "miniconda",
        "programs", "python", "pythonsoftwarefoundation", "pyenv",
        "ms-playwright", "playwright", "chromedriver", "selenium",
        "git", "github", "gitkraken", "sourcetree",
        "docker", "kubernetes", "vagrant", "virtualbox", "vmware",
        "spotify",
        "1password", "bitwarden", "keepass", "lastpass", "dashlane",
        "obsidian", "notion", "evernote", "onenote",
        "adobephotoshop", "adobe", "illustrator", "indesign", "premiere",
        "blender",
        "logitech", "razer", "corsair", "steelseries",
        "teams", "skype", "zoom", "webex",
        "crashpad", "crashpad_handler", "dumps", "minidump",
        "cache", "temp", "tmp", "logs", "crashreports",
        "nvidia corporation", "amd", "intel", "realtek",
        "mailbird",
        "oracle", "java", "javasoft", "openjdk",
        "sysinternals", "nirsoft",
        "lm studio", "lmstudio", "ollama", "kobold", "koboldai",
        "stable diffusion", "automatic1111", "comfyui", "invokeai",
        "stability", "midjourney", "openai",
        "comms", "vst", "vst3", "plugin", "plugins",
        "extension", "extensions", "addon", "addons",
        "obs", "obs-studio", "streamlabs", "xsplit", "vdo", "ninja",
        "captura", "sharex", "lightshot", "greenshot", "snipping",
        "audacity", "ocenaudio", "ocen audio", "flstudio", "fl studio",
        "ableton", "reaper", "bitwig", "cubase", "protools", "reason",
        "lmms", "studio one",
        "reallusion", "iclone", "character creator", "accuRIG", "actorcore",
        "maxon", "cinema 4d", "redshift",
        "foundry", "nuke", "mari", "katana", "modo",
        "pixologic", "zbrush",
        "allegorithmic", "substance",
        "autodesk", "maya", "3ds max", "motionbuilder", "mudbox",
        "sidefx", "houdini", "houdini engine",
        "epic games launcher", "unreal editor", "ue_", "ue4editor", "ue5editor",
        "daz3d", "poser", "hexagon", " bryce",
        "render", "renderer", "rendering", "octane", "redshift", "arnold",
        "keyshot", "vray", "corona",
    ];
    DENIED.iter().any(|d| lp == *d || lp.starts_with(d) || lp.contains(d))
}

fn is_likely_publisher(lp: &str) -> bool {
    if is_denied_publisher(lp) {
        return false;
    }
    const KNOWN_PUBLISHERS: &[&str] = &[
        "ubisoft", "ea", "electronic arts", "bethesda", "larian", "larian studios",
        "cd projekt", "cdprojekt", "square enix", "capcom", "konami", "namco",
        "bandai namco", "warn", "2k games", "sega", "activision", "blizzard",
        "riot", "epic", "rockstar", "paradox", "feral", "aspyr", "feral interactive",
        "frictional", "klei", "11 bit", "11bitstudios", "devolver", "annapurna",
        "playdead", "moon studios", "obsidian", "inxile", "stardock",
        "xbox", "microsoft games", "mojang", "minecraft",
        "valve", "steam", "gearbox", "2k", "firaxis", "petroglyph",
        "warframe", "digital extremes", "warhorse", "deep silver",
        "focus", "focus home", "focus_entertainment", "private division",
        "rockfish", "gaijin", "wargaming", "gameloft",
        "croteam", "supergiant", "saber", "id software", "machinegames",
        "playground", "forza", "halo", "gears", "turn 10",
        "moss", "polytron", "thatgamecompany", "team cherry", "hK",
        "hgames", "moonlight", "poncle", "no more robots",
        "stunlock", "fatshark", "thq", "thq nordic", "koch media",
        "playway", "all in! games", "movie games", "imgn.pro",
        "creality", "prusa", "prusament", "anycubic",
    ];
    if KNOWN_PUBLISHERS.iter().any(|p| lp == *p || lp.contains(p)) {
        return true;
    }
    let has_space = lp.contains(' ');
    let words: Vec<&str> = lp.split_whitespace().collect();
    if has_space && words.len() <= 4 && words.iter().all(|w| w.len() >= 3) {
        return true;
    }
    if lp.contains("game") || lp.contains("studio") || lp.contains("interactive") {
        return true;
    }
    false
}

fn is_denied_game_name(lname: &str) -> bool {
    const DENIED: &[&str] = &[
        "cache", "temp", "tmp", "logs", "log", "crash", "dumps", "minidump",
        "settings", "preferences", "prefs", "config", "configuration",
        "downloads", "download", "installer", "installers", "setup",
        "screenshot", "screenshots", "captures", "movies", "clips",
        "shader", "shaders", "pipeline", "pipelinecache",
        "backup", "backups", "old", "archive", "archives",
        "index", "tmp", "crashpad", "crashpad_handler",
        "launcher", "launcher-v2", "launcher2", "launcher64", "updater",
        "client", "service", "services", "agent", "daemon", "worker",
        "redist", "redistributables", "shared", "common", "crashrpt",
        "webcache", "blob_storage", "databases", "local storage",
        "sessionstorage", "cookies", "history",
        "easyanticheat", "eac", "anticheat",
        "manifest", "metadata", "build", "builds",
    ];
    if DENIED.iter().any(|d| lname == *d || lname.contains(d)) {
        return true;
    }
    let hex_chars: usize = lname.chars().filter(|c| c.is_ascii_hexdigit()).count();
    if hex_chars >= 24 && lname.chars().all(|c| c.is_ascii_hexdigit() || c.is_ascii_digit()) {
        return true;
    }
    if lname.len() <= 2 && lname.chars().all(|c| c.is_ascii_lowercase()) {
        return true;
    }
    false
}

fn classify_orphan(
    path: &Path,
    name: &str,
    publisher: &str,
    manifest: &[crate::model::ManifestEntry],
    installed: &[InstalledGame],
) -> Option<OrphanedFile> {
    let lname = name.to_lowercase();
    let (category, mut confidence): (FileCategory, u8) = match lname.as_str() {
        n if n.contains("save") || n.contains("slot") => (FileCategory::Saves, 75),
        n if n.ends_with("sav") || n.ends_with(".sav") => (FileCategory::Saves, 60),
        n if n.contains("profile") => (FileCategory::Settings, 50),
        _ => (FileCategory::Other, 30),
    };
    if lname.contains("cache") {
        confidence = 85;
    }
    let lpub = publisher;
    if lpub.contains("ubisoft") || lpub.contains("ea") || lpub.contains("bethesda")
        || lpub.contains("larian") || lpub.contains("rockstar")
        || lpub.contains("paradox") || lpub.contains("2k") || lpub.contains("capcom")
    {
        confidence = confidence.saturating_add(25);
    }
    let in_manifest = manifest.iter().any(|m| {
        m.name.to_lowercase() == lname
            || m.save_paths
                .iter()
                .any(|p: &String| p.to_lowercase().contains(&lname))
    });
    if in_manifest {
        confidence = confidence.saturating_add(20);
    }
    let source = guess_source(publisher, installed);
    if source.is_some() {
        confidence = confidence.saturating_add(10);
    }
    if confidence < 50 {
        return None;
    }
    let reason = if in_manifest {
        format!("Listed in Ludusavi manifest")
    } else if matches!(category, FileCategory::Saves) {
        format!("Save folder under {publisher}")
    } else {
        format!("Folder under {publisher} not linked to installed game")
    };
    Some(OrphanedFile {
        id: uuid::Uuid::new_v4().to_string(),
        game_hint: name.to_string(),
        category,
        path: path.to_path_buf(),
        size_bytes: 0,
        file_count: 0,
        reason,
        last_modified: None,
        confidence: confidence.min(100),
        source,
    })
}

fn guess_source(publisher: &str, installed: &[InstalledGame]) -> Option<GameSource> {
    for g in installed {
        if publisher.contains(&g.name.to_lowercase())
            || g.name.to_lowercase().contains(publisher)
        {
            return Some(g.source);
        }
    }
    None
}

fn dir_size_and_count(path: &Path) -> (u64, u32) {
    let mut total = 0u64;
    let mut count = 0u32;
    for entry in WalkDir::new(path).into_iter().flatten() {
        if entry.file_type().is_file() {
            count += 1;
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    (total, count)
}

fn collect_system_info() -> SystemInfo {
    let mut info = SystemInfo {
        hostname: hostname(),
        windows_version: current_windows_version(),
        ..Default::default()
    };
    if let Some(home) = directories::UserDirs::new().map(|u| u.home_dir().to_path_buf()) {
        if let Ok(Some(disk)) = get_disk_free_space(&home) {
            info.total_bytes = disk.0;
            info.free_bytes = disk.1;
        }
    }
    info
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

fn current_windows_version() -> String {
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion") {
        let product: String = key.get_value("ProductName").unwrap_or_default();
        let display: String = key.get_value("DisplayVersion").unwrap_or_default();
        if !product.is_empty() {
            return if display.is_empty() {
                product
            } else {
                format!("{product} ({display})")
            };
        }
    }
    "Windows".into()
}

#[cfg(windows)]
fn get_disk_free_space(path: &Path) -> std::io::Result<Option<(u64, u64)>> {
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut free_bytes_avail: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut free_bytes_total: u64 = 0;
    let result = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_bytes_avail,
            &mut total_bytes,
            &mut free_bytes_total,
        )
    };
    if result != 0 {
        Ok(Some((total_bytes, free_bytes_total)))
    } else {
        Ok(None)
    }
}

#[cfg(not(windows))]
fn get_disk_free_space(_path: &Path) -> std::io::Result<Option<(u64, u64)>> {
    Ok(None)
}

pub mod test_helpers {
    use super::*;
    pub fn is_denied_publisher_for_test(s: &str) -> bool {
        is_denied_publisher(&s.to_lowercase())
    }
    pub fn is_likely_publisher_for_test(s: &str) -> bool {
        is_likely_publisher(&s.to_lowercase())
    }
    pub fn is_denied_game_name_for_test(s: &str) -> bool {
        is_denied_game_name(&s.to_lowercase())
    }
    pub fn is_whitelisted_path_for_test(p: &Path, wl: &Whitelist) -> bool {
        is_whitelisted_path(p, wl)
    }
    pub fn human_size(n: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut v = n as f64;
        let mut i = 0;
        while v >= 1024.0 && i < UNITS.len() - 1 {
            v /= 1024.0;
            i += 1;
        }
        format!("{:.2} {}", v, UNITS[i])
    }
}
