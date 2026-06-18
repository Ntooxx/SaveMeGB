use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use std::env;

const KEY_PREFIX: &str = "SMGB";
const SEGMENT_LEN: usize = 8;
const VERSION: u8 = 1;
const PAYLOAD_LEN: usize = 9;
const SIG_FULL_LEN: usize = 64;
const TOTAL_BYTES: usize = PAYLOAD_LEN + SIG_FULL_LEN;
const B64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  keygen <identity>          Generate one key for an email/order-id");
        eprintln!("  keygen bulk <count>        Generate <count> keys as CSV (for Gumroad)");
        eprintln!("  keygen --generate          Generate a new Ed25519 keypair");
        eprintln!();
        eprintln!("  Set SMGB_PRIVATE_KEY env var or create a .secret file.");
        std::process::exit(1);
    }

    if args[1] == "--generate" {
        generate_keypair();
        return;
    }

    if args[1] == "bulk" {
        let count: u32 = args.get(2)
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);
        bulk_generate(count);
        return;
    }

    let identity = &args[1];
    let secret_hex = load_secret();
    let signing_key = load_signing_key(&secret_hex);
    let key = generate_key(&signing_key, identity);
    println!("{}", key);
    eprintln!("Identity: {}", identity);
    eprintln!("Key length: {} chars", key.len());
}

fn generate_keypair() {
    use rand::rngs::OsRng;
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let secret_hex = hex::encode(signing_key.to_bytes());
    let public_hex = hex::encode(signing_key.verifying_key().to_bytes());

    println!("Private key (keep secret): {}", secret_hex);
    println!("Public key  (bake into app): {}", public_hex);
    println!();
    println!("Build with:");
    println!("  set SMGB_PUBLIC_KEY={}", public_hex);
}

fn load_secret() -> String {
    if let Ok(val) = env::var("SMGB_PRIVATE_KEY") {
        let val = val.trim().to_string();
        if !val.is_empty() {
            return val;
        }
    }
    if let Ok(val) = std::fs::read_to_string(".secret") {
        let val = val.trim().to_string();
        if !val.is_empty() {
            return val;
        }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            let path = dir.join(".secret");
            if let Ok(val) = std::fs::read_to_string(&path) {
                let val = val.trim().to_string();
                if !val.is_empty() {
                    return val;
                }
            }
        }
    }
    eprintln!("ERROR: No private key found.");
    eprintln!("Set SMGB_PRIVATE_KEY env var or create a .secret file.");
    eprintln!("Generate a keypair with: keygen --generate");
    std::process::exit(1);
}

fn load_signing_key(secret_hex: &str) -> SigningKey {
    let secret_bytes: [u8; 32] = hex::decode(secret_hex)
        .expect("Private key must be 64 hex chars (32 bytes)")
        .try_into()
        .expect("Private key must be exactly 32 bytes");
    SigningKey::from_bytes(&secret_bytes)
}

fn generate_key(signing_key: &SigningKey, identity: &str) -> String {
    let mut payload = [0u8; PAYLOAD_LEN];
    payload[0] = VERSION;
    let identity_hash = Sha256::digest(identity.as_bytes());
    payload[1..9].copy_from_slice(&identity_hash[..8]);

    let signature = signing_key.sign(&payload);

    let mut combined = Vec::with_capacity(TOTAL_BYTES);
    combined.extend_from_slice(&payload);
    combined.extend_from_slice(&signature.to_bytes());

    let encoded = b64_encode(&combined);

    // Format with '~' every SEGMENT_LEN chars for readability
    let chars: Vec<char> = encoded.chars().collect();
    let mut segments = Vec::new();
    for chunk in chars.chunks(SEGMENT_LEN) {
        segments.push(chunk.iter().collect::<String>());
    }
    format!("{}-{}", KEY_PREFIX, segments.join("~"))
}

fn bulk_generate(count: u32) {
    let secret_hex = load_secret();
    let signing_key = load_signing_key(&secret_hex);

    // CSV header for Gumroad license keys
    println!("license_key");

    for i in 0..count {
        // Use UUID-like random identity so keys aren't tied to emails
        // This means no email verification — the key itself is proof of purchase
        let random_id = format!("bulk-{:08x}-{}", i, uuid::Uuid::new_v4());
        let key = generate_key(&signing_key, &random_id);
        println!("{}", key);
    }

    eprintln!("Generated {} keys. Upload this CSV to Gumroad (Product → License Keys).", count);
    eprintln!("Users won't need email verification — the key alone is proof of purchase.");
}
