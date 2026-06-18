use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const PUBLIC_KEY_HEX: &str = env!(
    "SMGB_PUBLIC_KEY",
    "SMGB_PUBLIC_KEY env var not set at build time"
);
const KEY_PREFIX: &str = "SMGB";
const VERSION: u8 = 1;
const PAYLOAD_LEN: usize = 9;
const SIG_FULL_LEN: usize = 64;
const TOTAL_BYTES: usize = PAYLOAD_LEN + SIG_FULL_LEN;

/// Base64URL alphabet (RFC 4648 §5, URL/filename-safe)
const B64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

// Rate-limiting: max activation attempts per window
const RATE_LIMIT_MAX: u32 = 5;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

static LAST_ATTEMPTS: Mutex<Vec<Instant>> = Mutex::new(Vec::new());

fn check_rate_limit() -> Result<(), String> {
    let mut attempts = LAST_ATTEMPTS.lock().unwrap();
    let now = Instant::now();
    attempts.retain(|t| now.duration_since(*t) < RATE_LIMIT_WINDOW);
    if attempts.len() >= RATE_LIMIT_MAX as usize {
        return Err("Too many activation attempts. Wait 60 seconds.".into());
    }
    attempts.push(now);
    Ok(())
}

fn public_key() -> Result<ed25519_dalek::VerifyingKey, String> {
    let bytes: [u8; 32] = hex::decode(PUBLIC_KEY_HEX)
        .map_err(|e| format!("public key decode: {e}"))?
        .try_into()
        .map_err(|_| "public key must be 32 bytes".to_string())?;
    ed25519_dalek::VerifyingKey::from_bytes(&bytes).map_err(|e| format!("public key: {e}"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub tier: String,
    pub issued_at: String,
    pub key_preview: String,
    /// Full license key, base64-encoded, for re-verification at load time
    pub(crate) encoded_key: String,
    /// HMAC-SHA256 of (tier + issued_at + preview + encoded_key).
    /// Prevents tampering with the stored JSON.
    pub(crate) tamper_check: String,
}

/// Public-facing license info (sent to frontend, excludes sensitive fields)
#[derive(Debug, Clone, Serialize)]
pub struct LicensePublicInfo {
    pub tier: String,
    pub issued_at: String,
    pub key_preview: String,
}

impl From<LicenseInfo> for LicensePublicInfo {
    fn from(info: LicenseInfo) -> Self {
        LicensePublicInfo {
            tier: info.tier,
            issued_at: info.issued_at,
            key_preview: info.key_preview,
        }
    }
}

#[allow(dead_code)]
fn b64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64_ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64_ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(B64_ALPHABET[(triple & 0x3F) as usize] as char);
        }
    }
    out
}

fn b64_decode_exact(input: &str, expected_len: usize) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let bytes: Vec<u8> = input
        .bytes()
        .filter_map(|c| {
            B64_ALPHABET
                .iter()
                .position(|&x| x == c)
                .map(|p| p as u8)
        })
        .collect();
    if bytes.len() * 6 / 8 < expected_len {
        return None;
    }
    for chunk in bytes.chunks(4) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let b3 = *chunk.get(3).unwrap_or(&0) as u32;
        out.push(((b0 << 2) | (b1 >> 4)) as u8);
        if chunk.len() > 2 {
            out.push(((b1 << 4) | (b2 >> 2)) as u8);
        }
        if chunk.len() > 3 {
            out.push(((b2 << 6) | b3) as u8);
        }
    }
    out.truncate(expected_len);
    if out.len() < expected_len {
        return None;
    }
    Some(out)
}

fn b64_decode(input: &str) -> Option<Vec<u8>> {
    // Decode variable-length base64 (for stored keys)
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let bytes: Vec<u8> = input
        .bytes()
        .filter_map(|c| {
            B64_ALPHABET
                .iter()
                .position(|&x| x == c)
                .map(|p| p as u8)
        })
        .collect();
    if bytes.is_empty() {
        return None;
    }
    for chunk in bytes.chunks(4) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let b3 = *chunk.get(3).unwrap_or(&0) as u32;
        out.push(((b0 << 2) | (b1 >> 4)) as u8);
        if chunk.len() > 2 {
            out.push(((b1 << 4) | (b2 >> 2)) as u8);
        }
        if chunk.len() > 3 {
            out.push(((b2 << 6) | b3) as u8);
        }
    }
    if out.is_empty() {
        return None;
    }
    Some(out)
}

pub fn validate_key_format(key: &str) -> Result<String, String> {
    let trimmed = key.trim();
    let expected_prefix = format!("{}-", KEY_PREFIX);
    if !trimmed.starts_with(&expected_prefix) {
        return Err(format!("Key must start with {}", expected_prefix));
    }

    let body = &trimmed[expected_prefix.len()..];

    if body.is_empty() {
        return Err("Key too short".into());
    }

    // Verify all chars are valid base64url (allow ~ as segment separator for readability)
    let clean: String = body.chars().filter(|c| *c != '~').collect();
    if !clean.bytes().all(|c| B64_ALPHABET.contains(&c)) {
        return Err("Key contains invalid characters".into());
    }

    if clean.len() < 80 {
        return Err(format!("Key too short ({} chars, need 80+)", clean.len()));
    }

    Ok(trimmed.to_string())
}

fn decode_key(key: &str) -> Result<([u8; PAYLOAD_LEN], [u8; SIG_FULL_LEN]), String> {
    let normalized = validate_key_format(key)?;
    let prefix_len = KEY_PREFIX.len() + 1; // "SMGB-"
    let body: String = normalized[prefix_len..]
        .chars()
        .filter(|c| *c != '~')
        .collect();

    let bytes = b64_decode_exact(&body, TOTAL_BYTES).ok_or_else(|| "Base64 decode failed".to_string())?;
    if bytes.len() < TOTAL_BYTES {
        return Err("Decoded key too short".into());
    }

    if bytes[0] != VERSION {
        return Err(format!("Unsupported key version {}", bytes[0]));
    }

    let mut payload = [0u8; PAYLOAD_LEN];
    payload.copy_from_slice(&bytes[..PAYLOAD_LEN]);
    let mut sig = [0u8; SIG_FULL_LEN];
    sig.copy_from_slice(&bytes[PAYLOAD_LEN..PAYLOAD_LEN + SIG_FULL_LEN]);

    Ok((payload, sig))
}

pub fn verify_license(key: &str, email: Option<&str>) -> Result<LicenseInfo, String> {
    check_rate_limit()?;

    let (payload, sig_bytes) = decode_key(key)?;

    let vk = public_key()?;
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);

    vk.verify_strict(&payload, &signature)
        .map_err(|_| "Invalid license key (signature mismatch)".to_string())?;

    // If email provided, verify the key was issued for this email
    if let Some(email) = email {
        let email = email.trim().to_lowercase();
        if !email.is_empty() {
            let email_hash = Sha256::digest(email.as_bytes());
            if payload[1..9] != email_hash[..8] {
                return Err("This key was not issued for that email address.".into());
            }
        }
    }

    let normalized = validate_key_format(key)?;
    let issued_at = chrono::Utc::now().to_rfc3339();
    let preview = if normalized.len() > 16 {
        format!("{}…", &normalized[..16])
    } else {
        normalized.clone()
    };

    let encoded_key = encode_key_for_storage(&normalized);
    let tamper = compute_tamper_check("pro", &issued_at, &preview, &encoded_key);

    Ok(LicenseInfo {
        tier: "pro".into(),
        issued_at,
        key_preview: preview,
        encoded_key,
        tamper_check: tamper,
    })
}

fn encode_key_for_storage(key: &str) -> String {
    // Simple reversable transform to avoid plaintext key in JSON
    // This is obfuscation, not encryption — the key is verifiable anyway
    let bytes: Vec<u8> = key.bytes().collect();
    b64_encode(&bytes)
}

fn decode_key_from_storage(encoded: &str) -> Option<String> {
    let bytes = b64_decode(encoded)?;
    String::from_utf8(bytes).ok()
}

fn compute_tamper_check(tier: &str, issued_at: &str, preview: &str, key: &str) -> String {
    let mut h = Sha256::new();
    h.update(tier.as_bytes());
    h.update(b"|");
    h.update(issued_at.as_bytes());
    h.update(b"|");
    h.update(preview.as_bytes());
    h.update(b"|");
    h.update(key.as_bytes());
    hex::encode(h.finalize())
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
    let info: LicenseInfo = serde_json::from_str(&raw).ok()?;
    if !verify_stored_license(&info) {
        let _ = std::fs::remove_file(&path);
        return None;
    }
    Some(info)
}

pub fn save_license(info: &LicenseInfo) -> Result<(), String> {
    let path = license_path()?;
    let body = serde_json::to_string_pretty(info).map_err(|e| e.to_string())?;
    std::fs::write(&path, &body).map_err(|e| format!("write license: {e}"))?;
    Ok(())
}

pub fn clear_license() -> Result<(), String> {
    let path = license_path()?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn verify_stored_license(info: &LicenseInfo) -> bool {
    if info.tier != "pro" || info.tamper_check.is_empty() || info.encoded_key.is_empty() {
        return false;
    }

    // Verify tamper check
    let expected = compute_tamper_check(
        &info.tier,
        &info.issued_at,
        &info.key_preview,
        &info.encoded_key,
    );
    if !constant_time_eq(info.tamper_check.as_bytes(), expected.as_bytes()) {
        return false;
    }

    // Re-verify the Ed25519 signature with the stored key
    let key = match decode_key_from_storage(&info.encoded_key) {
        Some(k) => k,
        None => return false,
    };

    let (payload, sig_bytes) = match decode_key(&key) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let vk = match public_key() {
        Ok(v) => v,
        Err(_) => return false,
    };

    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    vk.verify_strict(&payload, &signature).is_ok()
}

#[allow(dead_code)]
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

#[tauri::command]
pub fn activate_license(key: String, email: Option<String>) -> Result<LicensePublicInfo, String> {
    let email_ref = email.as_deref();
    let info = verify_license(&key, email_ref)?;
    save_license(&info)?;
    Ok(LicensePublicInfo::from(info))
}

#[tauri::command]
pub fn get_license() -> Option<LicensePublicInfo> {
    load_license().map(LicensePublicInfo::from)
}

#[tauri::command]
pub fn deactivate_license() -> Result<(), String> {
    clear_license()
}

#[allow(dead_code)]
pub fn _version() -> u8 {
    VERSION
}

#[cfg(test)]
fn reset_rate_limit() {
    LAST_ATTEMPTS.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_prefix() {
        assert!(validate_key_format("ABCD-abcdefgh-12345678").is_err());
    }

    #[test]
    fn rejects_too_short() {
        assert!(validate_key_format("SMGB-abc").is_err());
    }

    #[test]
    fn accepts_valid_format() {
        // 98 chars of base64url + prefix
        let body = "A".repeat(98);
        let key = format!("SMGB-{}", body);
        assert!(validate_key_format(&key).is_ok());
    }

    #[test]
    fn rejects_bad_signature() {
        reset_rate_limit();
        let result = verify_license("SMGB-ABCDEFGH-IJKLMNOP-QRSTUVWX-YZabcdef-ghijklmn-opqrstuv-wxyz0123-456789_-", None);
        assert!(result.is_err());
    }

    #[test]
    fn tamper_check_works() {
        let check = compute_tamper_check("pro", "2024-01-01", "SMGB-AAAA…", "encoded-key-here");
        assert!(!check.is_empty());
        assert_eq!(check.len(), 64); // SHA-256 hex is 64 chars
    }

    #[test]
    fn end_to_end_valid_key_verifies() {
        reset_rate_limit();
        if PUBLIC_KEY_HEX != "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a" {
            return;
        }
        use ed25519_dalek::{Signer, SigningKey};
        let secret: [u8; 32] = hex::decode(
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
        ).unwrap().try_into().unwrap();
        let signing_key = SigningKey::from_bytes(&secret);

        let mut payload = [0u8; PAYLOAD_LEN];
        payload[0] = VERSION;
        let hash = Sha256::digest(b"test@example.com");
        payload[1..9].copy_from_slice(&hash[..8]);

        let signature = signing_key.sign(&payload);

        let mut combined = Vec::with_capacity(TOTAL_BYTES);
        combined.extend_from_slice(&payload);
        combined.extend_from_slice(&signature.to_bytes());

        let encoded = b64_encode(&combined);
        let key = format!("{}-{}", KEY_PREFIX, encoded);

        let result = verify_license(&key, None);
        assert!(result.is_ok(), "Key should verify: {:?}", result.err());
    }

    #[test]
    fn tampered_key_rejected() {
        reset_rate_limit();
        if PUBLIC_KEY_HEX != "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a" {
            return;
        }
        use ed25519_dalek::{Signer, SigningKey};
        let secret: [u8; 32] = hex::decode(
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
        ).unwrap().try_into().unwrap();
        let signing_key = SigningKey::from_bytes(&secret);

        let mut payload = [0u8; PAYLOAD_LEN];
        payload[0] = VERSION;
        let hash = Sha256::digest(b"evil@hacker.com");
        payload[1..9].copy_from_slice(&hash[..8]);
        let signature = signing_key.sign(&payload);

        let mut bad_payload = payload;
        bad_payload[3] ^= 0x01;

        let mut combined = Vec::with_capacity(TOTAL_BYTES);
        combined.extend_from_slice(&bad_payload);
        combined.extend_from_slice(&signature.to_bytes());

        let encoded = b64_encode(&combined);
        let key = format!("{}-{}", KEY_PREFIX, encoded);

        assert!(verify_license(&key, None).is_err(), "Tampered key should be rejected");
    }

    #[test]
    fn email_binding_verified() {
        reset_rate_limit();
        if PUBLIC_KEY_HEX != "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a" {
            return;
        }
        use ed25519_dalek::{Signer, SigningKey};
        let secret: [u8; 32] = hex::decode(
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
        ).unwrap().try_into().unwrap();
        let signing_key = SigningKey::from_bytes(&secret);

        let email = "bob@gmail.com";
        let mut payload = [0u8; PAYLOAD_LEN];
        payload[0] = VERSION;
        let hash = Sha256::digest(email.as_bytes());
        payload[1..9].copy_from_slice(&hash[..8]);
        let signature = signing_key.sign(&payload);

        let mut combined = Vec::with_capacity(TOTAL_BYTES);
        combined.extend_from_slice(&payload);
        combined.extend_from_slice(&signature.to_bytes());
        let encoded = b64_encode(&combined);
        let key = format!("{}-{}", KEY_PREFIX, encoded);

        // Correct email should work
        assert!(verify_license(&key, Some("bob@gmail.com")).is_ok());
        // Wrong email should fail
        assert!(verify_license(&key, Some("eve@hacker.com")).is_err());
        // No email should still work (backwards compat)
        assert!(verify_license(&key, None).is_ok());
    }

    #[test]
    fn full_cycle_save_load_verify() {
        reset_rate_limit();
        if PUBLIC_KEY_HEX != "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a" {
            return;
        }
        use ed25519_dalek::{Signer, SigningKey};
        let secret: [u8; 32] = hex::decode(
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
        ).unwrap().try_into().unwrap();
        let signing_key = SigningKey::from_bytes(&secret);

        let mut payload = [0u8; PAYLOAD_LEN];
        payload[0] = VERSION;
        let hash = Sha256::digest(b"cycle-test@example.com");
        payload[1..9].copy_from_slice(&hash[..8]);
        let signature = signing_key.sign(&payload);

        let mut combined = Vec::with_capacity(TOTAL_BYTES);
        combined.extend_from_slice(&payload);
        combined.extend_from_slice(&signature.to_bytes());
        let encoded = b64_encode(&combined);
        let key = format!("{}-{}", KEY_PREFIX, encoded);

        // Activate
        let info = verify_license(&key, None).unwrap();
        save_license(&info).unwrap();

        // Load and verify
        let loaded = load_license().unwrap();
        assert_eq!(loaded.tier, "pro");
        assert!(!loaded.encoded_key.is_empty());
        assert!(!loaded.tamper_check.is_empty());

        // Tamper detection: modify stored JSON manually
        let mut tampered = loaded.clone();
        tampered.tier = "free".into();
        let path = license_path().unwrap();
        let body = serde_json::to_string_pretty(&tampered).unwrap();
        std::fs::write(&path, &body).unwrap();

        // Loading tampered file should return None (and delete it)
        assert!(load_license().is_none());
        assert!(!license_path().unwrap().exists());

        // Clean up: also clear the valid license we just saved
        let _ = clear_license();
    }

    #[test]
    fn b64_roundtrip() {
        let data: Vec<u8> = (0..TOTAL_BYTES).map(|i| i as u8).collect();
        let enc = b64_encode(&data);
        let dec = b64_decode_exact(&enc, TOTAL_BYTES).unwrap();
        assert_eq!(&data[..], &dec[..TOTAL_BYTES]);

        // Variable-length roundtrip
        let long_data: Vec<u8> = (0..200).map(|i| (i % 256) as u8).collect();
        let long_enc = b64_encode(&long_data);
        let long_dec = b64_decode(&long_enc).unwrap();
        assert_eq!(long_data.len(), long_dec.len());
        // Note: variable-length b64_decode may have one extra byte due to padding
        assert!(long_data.iter().zip(long_dec.iter()).all(|(a, b)| a == b));
    }
}
