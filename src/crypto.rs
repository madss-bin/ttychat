use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{SigningKey, VerifyingKey};
use std::{fs, path::PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn load_or_generate(key_path: &PathBuf) -> Result<(SigningKey, bool)> {
    if key_path.exists() {
        let raw = fs::read(key_path)?;
        let seed: [u8; 32] = raw
            .try_into()
            .map_err(|_| anyhow::anyhow!("Key file is corrupt (expected 32 bytes)"))?;
        let signing = SigningKey::from_bytes(&seed);
        Ok((signing, false))
    } else {
        let signing_key = generate_and_save(key_path)?;
        Ok((signing_key, true))
    }
}

pub fn generate_and_save(key_path: &PathBuf) -> Result<SigningKey> {
    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let seed = signing_key.to_bytes();

    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(key_path, seed)?;

    #[cfg(unix)]
    {
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(key_path, perms)?;
    }

    Ok(signing_key)
}

pub fn sign_nonce(signing_key: &SigningKey, nonce_b64: &str) -> Result<String> {
    use ed25519_dalek::Signer;
    let nonce_bytes = B64.decode(nonce_b64)?;
    let sig = signing_key.sign(&nonce_bytes);
    Ok(B64.encode(sig.to_bytes()))
}

pub fn pubkey_b64(signing_key: &SigningKey) -> String {
    let verifying: VerifyingKey = signing_key.verifying_key();
    B64.encode(verifying.as_bytes())
}

pub fn cmd_gen(username: Option<&str>) -> Result<()> {
    let un = username.unwrap_or("default");
    let key_path = crate::config::Config::key_path(un);
    let (signing_key, is_new) = load_or_generate(&key_path)?;
    let pub_b64 = pubkey_b64(&signing_key);

    if is_new {
        eprintln!("[!] Generated new ed25519 identity.");
        eprintln!("    Saved to: {}", key_path.display());
    } else {
        eprintln!("[*] Loaded existing identity.");
    }
    println!("\nPublic Key (base64):\n  {}", pub_b64);
    println!("\nTo add yourself, ask an admin to run:");
    println!("  ttychatctl add <pubkey> -user <username>");
    Ok(())
}

pub fn cmd_reset(username: Option<&str>) -> Result<()> {
    if let Some(un) = username {
        let key_path = crate::config::Config::key_path(un);
        if key_path.exists() {
            std::fs::remove_file(&key_path)?;
            eprintln!("[*] Deleted identity key for user '{}' at {}", un, key_path.display());
        } else {
            eprintln!("[!] No identity key found for user '{}' at {}", un, key_path.display());
        }
    } else {
        let dir = crate::config::Config::config_dir();
        eprintln!("[!] Please specify a username to reset: ttychat reset <username>");
        eprintln!("    Keys are stored in: {}", dir.display());
    }
    Ok(())
}
