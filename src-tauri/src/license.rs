use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::PathBuf;

const HMAC_KEY_HEX: &str = "0c1a2b3c4d5e6f708192a3b4c5d6e7f00123456789abcdef0123456789abcdef";
const LICENSE_VERSION: u8 = 1;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub tier: String,
    pub issued_at: String,
    pub key_preview: String,
}

pub fn validate_key_format(key: &str) -> Result<String, String> {
    let trimmed = key.trim().to_uppercase().replace(' ', "");
    let parts: Vec<&str> = trimmed.split('-').collect();
    if parts.len() != 5 || parts[0] != "SMGB" {
        return Err("Key must look like SMGB-XXXX-XXXX-XXXX-XXXX".into());
    }
    for part in &parts[1..] {
        if part.len() != 4 || !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err("Key segments must be 4 alphanumeric chars".into());
        }
    }
    Ok(trimmed)
}

pub fn verify_license(key: &str) -> Result<LicenseInfo, String> {
    let normalized = validate_key_format(key)?;
    let body = &normalized[5..];
    let provided = &body[17..];
    let payload = &body[..17];

    let mut mac = HmacSha256::new_from_slice(
        hex::decode(HMAC_KEY_HEX).map_err(|e| format!("key decode: {e}"))?.as_slice(),
    )
    .map_err(|e| format!("hmac init: {e}"))?;
    mac.update(payload.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    if !constant_time_eq(provided.to_lowercase().as_bytes(), expected.to_lowercase().as_bytes()) {
        return Err("Invalid license key (signature mismatch)".into());
    }

    let issued_at = chrono::Utc::now().to_rfc3339();
    let preview = format!("{}****-****-****", &normalized[..9]);
    Ok(LicenseInfo {
        tier: "pro".into(),
        issued_at,
        key_preview: preview,
    })
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn license_path() -> Result<PathBuf, String> {
    let dir = directories::ProjectDirs::from("com", "savemegb", "SaveMeGB")
        .ok_or_else(|| "could not resolve config dir".to_string())?
        .config_dir()
        .to_path_buf();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create config dir: {e}"))?;
    Ok(dir.join("license.json"))
}

pub fn load_license() -> Option<LicenseInfo> {
    let path = license_path().ok()?;
    let raw = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn save_license(info: &LicenseInfo) -> Result<(), String> {
    let path = license_path()?;
    let body = serde_json::to_string_pretty(info).map_err(|e| e.to_string())?;
    std::fs::write(&path, body).map_err(|e| e.to_string())
}

pub fn clear_license() -> Result<(), String> {
    let path = license_path()?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn activate_license(key: String) -> Result<LicenseInfo, String> {
    let info = verify_license(&key)?;
    save_license(&info)?;
    Ok(info)
}

#[tauri::command]
pub fn get_license() -> Option<LicenseInfo> {
    load_license()
}

#[tauri::command]
pub fn deactivate_license() -> Result<(), String> {
    clear_license()
}

#[allow(dead_code)]
pub fn _version() -> u8 {
    LICENSE_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_prefix() {
        assert!(validate_key_format("ABCD-1234-1234-1234-1234").is_err());
    }

    #[test]
    fn rejects_wrong_segment_count() {
        assert!(validate_key_format("SMGB-1234-1234").is_err());
    }

    #[test]
    fn rejects_wrong_segment_length() {
        assert!(validate_key_format("SMGB-123-1234-1234-1234").is_err());
    }

    #[test]
    fn accepts_valid_format() {
        let key = "SMGB-AAAA-BBBB-CCCC-DDDD";
        assert!(validate_key_format(key).is_ok());
    }

    #[test]
    fn rejects_invalid_signature() {
        let result = verify_license("SMGB-AAAA-BBBB-CCCC-FFFF");
        assert!(result.is_err());
    }
}
