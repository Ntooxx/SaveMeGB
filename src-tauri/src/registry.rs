use crate::model::{GameSource, InstalledGame};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use winreg::enums::*;
use winreg::RegKey;

const STEAM_REG: &str = r"Software\Valve\Steam";
const EPIC_REG: &str = r"Software\Epic Games\EOS";
const EPIC_LAUNCHER_REG: &str = r"Software\Epic Games\Unreal Engine\1.0";
const GOG_REG: &str = r"Software\GOG.com\GalaxyClient\paths";
const BLIZZARD_REG: &str = r"Software\Blizzard Entertainment\Battle.net\Launch Options";
const EA_REG: &str = r"Software\Electronic Arts\EA Core";
const UBISOFT_REG: &str = r"Software\Ubisoft\Launcher";
const RIOT_REG: &str = r"Software\Riot Games\Riot Client";

pub fn scan_all() -> Result<Vec<InstalledGame>> {
    let mut games = Vec::new();
    for result in [
        scan_steam(),
        scan_epic(),
        scan_xbox(),
        scan_gog(),
        scan_battlenet(),
        scan_riot(),
        scan_ea(),
        scan_ubisoft(),
        scan_manual(),
    ] {
        match result {
            Ok(mut g) => games.append(&mut g),
            Err(e) => log::warn!("launcher scan failed: {e}"),
        }
    }
    Ok(games)
}

fn open_steam_key() -> Result<RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(STEAM_REG).context("Steam not installed")
}

fn read_library_paths(steam_path: &Path) -> Vec<PathBuf> {
    let mut paths = vec![steam_path.join("steamapps")];
    let vdf_path = steam_path.join("steamapps").join("libraryfolders.vdf");
    let content = match std::fs::read_to_string(&vdf_path) {
        Ok(c) => c,
        Err(_) => return paths,
    };
    let vdf = match keyvalues_parser::parse(&content) {
        Ok(v) => v,
        Err(_) => return paths,
    };
    let Some(obj) = vdf.value.get_obj() else {
        return paths;
    };
    let candidates: &[&str] = &["libraryfolders", "LibraryFolders"];
    let mut entries = None;
    for key in candidates {
        if let Some(e) = obj.get(*key) {
            entries = Some(e);
            break;
        }
    }
    let Some(entries) = entries else {
        return paths;
    };
    for value in entries {
        let Some(lf) = value.get_obj() else { continue };
        if let Some(paths_arr) = lf.get("path") {
            for p in paths_arr {
                if let Some(s) = p.get_str() {
                    let lib = PathBuf::from(s);
                    if lib.exists() {
                        paths.push(lib.join("steamapps"));
                    }
                }
            }
        }
        for (key, vals) in lf.iter() {
            if key.parse::<u32>().is_ok() {
                for v in vals {
                    if let Some(s) = v.get_str() {
                        let lib = PathBuf::from(s);
                        if lib.exists() {
                            paths.push(lib.join("steamapps"));
                        }
                    }
                }
            }
        }
    }
    paths
}

fn read_app_manifest(steamapps: &Path) -> HashMap<String, (String, String, String)> {
    let mut out = HashMap::new();
    let entries = match std::fs::read_dir(steamapps) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("acf") {
            continue;
        }
        let content = match std::fs::read_to_string(&p) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let vdf = match keyvalues_parser::parse(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let Some(obj) = vdf.value.get_obj() else { continue };
        let appid = first_value_str(obj, "appid").map(String::from);
        let name = first_value_str(obj, "name").map(String::from);
        let installdir = first_value_str(obj, "installdir").map(String::from);
        if let (Some(appid), Some(name), Some(installdir)) = (appid, name, installdir.clone()) {
            out.insert(installdir.clone(), (appid, name, installdir));
        }
    }
    out
}

fn first_value_str(obj: &keyvalues_parser::Obj<'_>, key: &str) -> Option<String> {
    let vals = obj.get(key)?;
    let v = vals.first()?;
    if let Some(s) = v.get_str() {
        return Some(s.to_string());
    }
    None
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    for entry in WalkDir::new(path).into_iter().flatten() {
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn estimate_save_paths(name: &str, _steamapps: &Path, app_id: Option<&str>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let Some(home) = directories::UserDirs::new().map(|u| u.home_dir().to_path_buf()) else {
        return paths;
    };
    let appdata = home.join("AppData");
    let candidates = [
        appdata.join("Local").join(name),
        appdata.join("LocalLow").join(name),
        appdata.join("Roaming").join(name),
        home.join("Documents").join("My Games").join(name),
        home.join("Saved Games").join(name),
        home.join("Documents").join(name),
    ];
    for c in candidates {
        if c.exists() {
            paths.push(c);
        }
    }
    if let Some(id) = app_id {
        let id_paths = [
            home.join("Documents").join("My Games").join(format!("id{id}_{name}")),
            home.join("AppData").join("Local").join(format!("{name}_data")),
        ];
        for c in id_paths {
            if c.exists() {
                paths.push(c);
            }
        }
    }
    paths
}

pub fn scan_steam() -> Result<Vec<InstalledGame>> {
    let steam_key = open_steam_key()?;
    let steam_path: String = steam_key
        .get_value("SteamPath")
        .context("SteamPath value missing")?;
    let steam_path = PathBuf::from(steam_path);
    if !steam_path.exists() {
        return Ok(vec![]);
    }
    let library_paths = read_library_paths(&steam_path);
    let mut games = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for lib in library_paths {
        let manifests = read_app_manifest(&lib);
        let common = lib.join("common");
        if !common.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&common)?.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if !seen.insert(dir_name.clone()) {
                continue;
            }
            let path = entry.path();
            let (app_id, display_name, _installdir) = manifests
                .get(&dir_name)
                .map(|(id, n, i)| (Some(id.clone()), n.clone(), i.clone()))
                .unwrap_or_else(|| (None, dir_name.clone(), dir_name.clone()));
            let size = dir_size(&path);
            let estimated = estimate_save_paths(&display_name, &lib, app_id.as_deref());
            games.push(InstalledGame {
                id: format!("steam:{}", app_id.clone().unwrap_or_else(|| dir_name.clone())),
                name: display_name,
                source: GameSource::Steam,
                install_path: path,
                app_id,
                size_bytes: size,
                estimated_save_paths: estimated,
            });
        }
    }
    Ok(games)
}

pub fn scan_epic() -> Result<Vec<InstalledGame>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let mut games = Vec::new();
    let mut manifests_dir: Option<PathBuf> = None;
    if let Ok(key) = hkcu.open_subkey(EPIC_REG) {
        for value_name in ["ModSdkMetadata", "ManifestsDir", "ManifestDirectory"] {
            if let Ok(p) = key.get_value::<String, _>(value_name) {
                let candidate = PathBuf::from(&p);
                let base = if candidate.is_file() {
                    candidate.parent().map(|p| p.to_path_buf())
                } else {
                    Some(candidate)
                };
                if let Some(b) = base {
                    if b.exists() {
                        manifests_dir = Some(b);
                        break;
                    }
                }
            }
        }
    }
    if manifests_dir.is_none() {
        if let Ok(key) = hkcu.open_subkey(EPIC_LAUNCHER_REG) {
            if let Ok(p) = key.get_value::<String, _>("INSTALLDIR") {
                manifests_dir = Some(PathBuf::from(p));
            }
        }
    }
    let Some(manifests_dir) = manifests_dir else { return Ok(games) };
    if !manifests_dir.exists() {
        return Ok(games);
    }
    for entry in WalkDir::new(&manifests_dir)
        .max_depth(4)
        .into_iter()
        .flatten()
    {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("item") {
            continue;
        }
        let content = match std::fs::read_to_string(p) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let name = json
            .get("DisplayName")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let install_loc = json
            .get("InstallLocation")
            .and_then(|v| v.as_str())
            .map(PathBuf::from);
        let app_name = json
            .get("AppName")
            .and_then(|v| v.as_str())
            .map(String::from);
        if let Some(loc) = install_loc {
            if loc.exists() {
                let size = dir_size(&loc);
                let estimated = estimate_save_paths(&name, &PathBuf::new(), None);
                games.push(InstalledGame {
                    id: format!("epic:{}", app_name.clone().unwrap_or_else(|| name.clone())),
                    name: if name.is_empty() {
                        loc.file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default()
                    } else {
                        name.clone()
                    },
                    source: GameSource::Epic,
                    install_path: loc.clone(),
                    app_id: app_name,
                    size_bytes: size,
                    estimated_save_paths: estimated,
                });
            }
        }
    }
    Ok(games)
}

pub fn scan_xbox() -> Result<Vec<InstalledGame>> {
    let mut games = Vec::new();
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(home) = directories::UserDirs::new().map(|u| u.home_dir().to_path_buf()) {
        candidates.push(home.join("XboxGames"));
    }
    candidates.push(PathBuf::from(r"C:\XboxGames"));
    for base in candidates {
        if !base.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&base)?.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let size = dir_size(&path);
            games.push(InstalledGame {
                id: format!("xbox:{name}"),
                name: name.clone(),
                source: GameSource::Xbox,
                install_path: path,
                app_id: None,
                size_bytes: size,
                estimated_save_paths: vec![],
            });
        }
    }
    Ok(games)
}

pub fn scan_gog() -> Result<Vec<InstalledGame>> {
    let mut games = Vec::new();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey(GOG_REG) else {
        return Ok(games);
    };
    let Ok(client_path) = key.get_value::<String, _>("client") else {
        return Ok(games);
    };
    let games_dir = PathBuf::from(client_path).join("Games");
    if !games_dir.exists() {
        return Ok(games);
    }
    for entry in std::fs::read_dir(&games_dir)?.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let size = dir_size(&path);
        games.push(InstalledGame {
            id: format!("gog:{name}"),
            name: name.clone(),
            source: GameSource::GOG,
            install_path: path,
            app_id: None,
            size_bytes: size,
            estimated_save_paths: vec![],
        });
    }
    Ok(games)
}

pub fn scan_battlenet() -> Result<Vec<InstalledGame>> {
    let mut games = Vec::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let mut install_dirs: Vec<PathBuf> = Vec::new();
    for key in [
        hklm.open_subkey(BLIZZARD_REG),
        hkcu.open_subkey(BLIZZARD_REG),
    ] {
        if let Ok(k) = key {
            for value_name in ["InstallPath", "GameDir", "Install"] {
                if let Ok(p) = k.get_value::<String, _>(value_name) {
                    let pb = PathBuf::from(p);
                    if pb.exists() && !install_dirs.contains(&pb) {
                        install_dirs.push(pb);
                    }
                }
            }
        }
    }
    for probe in [
        PathBuf::from(r"C:\Program Files (x86)\Battle.net"),
        PathBuf::from(r"C:\Program Files\Battle.net"),
    ] {
        if probe.exists() && !install_dirs.contains(&probe) {
            install_dirs.push(probe);
        }
    }
    for base in &install_dirs {
        let candidates = [base.clone(), base.join("Games"), base.join("GameData")];
        for dir in candidates {
            if !dir.exists() {
                continue;
            }
            let entries = match std::fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name == "_retail_" {
                    continue;
                }
                let size = dir_size(&path);
                games.push(InstalledGame {
                    id: format!("battlenet:{name}"),
                    name: name.clone(),
                    source: GameSource::Manual,
                    install_path: path,
                    app_id: None,
                    size_bytes: size,
                    estimated_save_paths: vec![],
                });
            }
        }
    }
    Ok(games)
}

pub fn scan_riot() -> Result<Vec<InstalledGame>> {
    let mut games = Vec::new();
    let candidates = [
        PathBuf::from(r"C:\Riot Games"),
        PathBuf::from(r"C:\Program Files\Riot Games"),
        PathBuf::from(r"C:\Program Files (x86)\Riot Games"),
    ];
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey(RIOT_REG) {
        if let Ok(p) = key.get_value::<String, _>("InstallPath") {
            let pb = PathBuf::from(p);
            if pb.exists() && !candidates.contains(&pb) {
                scan_riot_dir(&pb, &mut games);
            }
        }
    }
    for c in candidates {
        if c.exists() {
            scan_riot_dir(&c, &mut games);
        }
    }
    Ok(games)
}

fn scan_riot_dir(base: &Path, games: &mut Vec<InstalledGame>) {
    let entries = match std::fs::read_dir(base) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "Riot Client" || name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        if !path.join("Game").exists() && !path.join("Engine").exists() && !path.join("data").exists() {
            continue;
        }
        let size = dir_size(&path);
        games.push(InstalledGame {
            id: format!("riot:{name}"),
            name: name.clone(),
            source: GameSource::Manual,
            install_path: path,
            app_id: None,
            size_bytes: size,
            estimated_save_paths: vec![],
        });
    }
}

pub fn scan_ea() -> Result<Vec<InstalledGame>> {
    let mut games = Vec::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut install_paths: Vec<PathBuf> = Vec::new();
    if let Ok(key) = hklm.open_subkey(EA_REG) {
        for value_name in ["InstallDir", "LauncherPath", "EADesktopPath"] {
            if let Ok(p) = key.get_value::<String, _>(value_name) {
                let pb = PathBuf::from(p);
                if pb.exists() && !install_paths.contains(&pb) {
                    install_paths.push(pb);
                }
            }
        }
    }
    let candidates = [
        PathBuf::from(r"C:\Program Files\Electronic Arts"),
        PathBuf::from(r"C:\Program Files (x86)\Electronic Arts"),
    ];
    for base in install_paths.iter().chain(candidates.iter()) {
        if !base.exists() {
            continue;
        }
        let entries = match std::fs::read_dir(base) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "EA Desktop" || name == "EA Core" || name.starts_with('.') {
                continue;
            }
            let path = entry.path();
            let size = dir_size(&path);
            games.push(InstalledGame {
                id: format!("ea:{name}"),
                name: name.clone(),
                source: GameSource::Manual,
                install_path: path,
                app_id: None,
                size_bytes: size,
                estimated_save_paths: vec![],
            });
        }
    }
    Ok(games)
}

pub fn scan_ubisoft() -> Result<Vec<InstalledGame>> {
    let mut games = Vec::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut install_paths: Vec<PathBuf> = Vec::new();
    if let Ok(key) = hklm.open_subkey(UBISOFT_REG) {
        for value_name in ["InstallDir", "exePath"] {
            if let Ok(p) = key.get_value::<String, _>(value_name) {
                let pb = PathBuf::from(p);
                if pb.exists() && !install_paths.contains(&pb) {
                    install_paths.push(pb);
                }
            }
        }
    }
    let candidates = [
        PathBuf::from(r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\games"),
        PathBuf::from(r"C:\Program Files\Ubisoft\Ubisoft Game Launcher\games"),
        PathBuf::from(r"C:\Ubisoft Game Launcher\games"),
    ];
    for base in install_paths.iter().chain(candidates.iter()) {
        if !base.exists() {
            continue;
        }
        let entries = match std::fs::read_dir(base) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            let size = dir_size(&path);
            games.push(InstalledGame {
                id: format!("ubisoft:{name}"),
                name: name.clone(),
                source: GameSource::Manual,
                install_path: path,
                app_id: None,
                size_bytes: size,
                estimated_save_paths: vec![],
            });
        }
    }
    Ok(games)
}

pub fn scan_manual() -> Result<Vec<InstalledGame>> {
    let mut games = Vec::new();
    let candidates = [
        PathBuf::from(r"C:\Games"),
        PathBuf::from(r"D:\Games"),
        PathBuf::from(r"E:\Games"),
        PathBuf::from(r"C:\GOG Games"),
        PathBuf::from(r"D:\GOG Games"),
    ];
    for base in candidates {
        if !base.exists() {
            continue;
        }
        let entries = match std::fs::read_dir(&base) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if !looks_like_game(&path) {
                continue;
            }
            let size = dir_size(&path);
            games.push(InstalledGame {
                id: format!("manual:{name}"),
                name: name.clone(),
                source: GameSource::Manual,
                install_path: path,
                app_id: None,
                size_bytes: size,
                estimated_save_paths: vec![],
            });
        }
    }
    Ok(games)
}

fn looks_like_game(dir: &Path) -> bool {
    let markers = [
        "game.exe", "Game.exe", "start.exe", "launcher.exe", "play.exe",
        "run.exe", "engine.exe", "unityengine.dll", "ue4.exe", "ue5.exe",
        "game.ico", "data", "Content", "Binaries", "Assets",
    ];
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return false,
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if markers.iter().any(|m| name.eq_ignore_ascii_case(m) || name.eq_ignore_ascii_case(&m[..m.len() - 4])) {
            return true;
        }
    }
    for entry in WalkDir::new(dir).max_depth(3).into_iter().flatten() {
        if entry.file_type().is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".exe") {
                return true;
            }
        }
    }
    false
}
