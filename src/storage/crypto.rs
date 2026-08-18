//! Local credential masking and API key obfuscation.
//!
//! Prevents API keys from casual plaintext exposure in dotfiles and JSON configuration
//! using reversible machine-derived XOR masking (`enc:v1:<base64>`). Note that this is UI masking
//! against casual inspection, not hardware-backed cryptographic storage.

use std::fs;

const PREFIX: &str = "enc:v1:";
const APP_SALT: &[u8] = b"quicky_notes_secure_local_salt_v1";

/// Derives a machine-specific key using /etc/machine-id or hostname fallback.
fn get_machine_key() -> Vec<u8> {
    let mut key = Vec::from(APP_SALT);
    if let Ok(machine_id) = fs::read_to_string("/etc/machine-id") {
        key.extend_from_slice(machine_id.trim().as_bytes());
    } else if let Ok(hostname) = fs::read_to_string("/etc/hostname") {
        key.extend_from_slice(hostname.trim().as_bytes());
    } else if let Ok(user) = std::env::var("USER") {
        key.extend_from_slice(user.as_bytes());
    }
    key
}

/// Simple standard Base64 encoder (RFC 4648).
fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len() * 4 / 3 + 4);
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as usize;
        let b1 = if i + 1 < data.len() {
            data[i + 1] as usize
        } else {
            0
        };
        let b2 = if i + 2 < data.len() {
            data[i + 2] as usize
        } else {
            0
        };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(CHARSET[(triple >> 18) & 0x3F] as char);
        out.push(CHARSET[(triple >> 12) & 0x3F] as char);
        if i + 1 < data.len() {
            out.push(CHARSET[(triple >> 6) & 0x3F] as char);
        } else {
            out.push('=');
        }
        if i + 2 < data.len() {
            out.push(CHARSET[triple & 0x3F] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

/// Simple standard Base64 decoder.
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let mut val = 0u32;
    let mut valb = -8;
    let mut out = Vec::new();
    for ch in input.chars() {
        if ch == '=' {
            break;
        }
        let v = match ch {
            'A'..='Z' => ch as u8 - b'A',
            'a'..='z' => ch as u8 - b'a' + 26,
            '0'..='9' => ch as u8 - b'0' + 52,
            '+' => 62,
            '/' => 63,
            _ => continue,
        };
        val = (val << 6) | (v as u32);
        valb += 6;
        if valb >= 0 {
            out.push(((val >> valb) & 0xFF) as u8);
            valb -= 8;
        }
    }
    Some(out)
}

/// Obfuscates a plaintext API key for safe disk storage.
pub fn obfuscate_key(plaintext: &str) -> String {
    let clean = plaintext.trim();
    if clean.is_empty() {
        return String::new();
    }
    if clean.starts_with(PREFIX) {
        return clean.to_string();
    }

    let key = get_machine_key();
    let plain_bytes = clean.as_bytes();
    let nonce: u32 = 0xA5C3_91E7 ^ (plain_bytes.len() as u32);

    let mut payload = Vec::with_capacity(4 + plain_bytes.len());
    payload.extend_from_slice(&nonce.to_le_bytes());

    for (i, &b) in plain_bytes.iter().enumerate() {
        let key_byte = key[i % key.len()];
        let nonce_byte = ((nonce >> ((i % 4) * 8)) & 0xFF) as u8;
        payload.push(b ^ key_byte ^ nonce_byte);
    }

    format!("{}{}", PREFIX, base64_encode(&payload))
}

/// Deobfuscates a stored credential string back to plaintext.
///
/// If the string is not encrypted (`enc:v1:...`), returns the string as-is for backward compatibility.
pub fn deobfuscate_key(stored: &str) -> String {
    let clean = stored.trim();
    if clean.is_empty() {
        return String::new();
    }
    if !clean.starts_with(PREFIX) {
        return clean.to_string();
    }

    let raw_b64 = &clean[PREFIX.len()..];
    let Some(payload) = base64_decode(raw_b64) else {
        return String::new();
    };

    if payload.len() < 5 {
        return String::new();
    }

    let nonce_bytes: [u8; 4] = [payload[0], payload[1], payload[2], payload[3]];
    let nonce = u32::from_le_bytes(nonce_bytes);
    let ciphertext = &payload[4..];
    let key = get_machine_key();

    let mut plain_bytes = Vec::with_capacity(ciphertext.len());
    for (i, &b) in ciphertext.iter().enumerate() {
        let key_byte = key[i % key.len()];
        let nonce_byte = ((nonce >> ((i % 4) * 8)) & 0xFF) as u8;
        plain_bytes.push(b ^ key_byte ^ nonce_byte);
    }

    String::from_utf8(plain_bytes).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obfuscation_roundtrip() {
        let original = "sk-proj-1234567890abcdef-TEST-KEY";
        let obfuscated = obfuscate_key(original);
        assert!(obfuscated.starts_with(PREFIX));
        assert_ne!(obfuscated, original);

        let restored = deobfuscate_key(&obfuscated);
        assert_eq!(restored, original);
    }

    #[test]
    fn test_legacy_plaintext_compatibility() {
        let legacy = "sk-plain-old-key-1234";
        let result = deobfuscate_key(legacy);
        assert_eq!(result, legacy);
    }

    #[test]
    fn test_empty_key() {
        assert_eq!(obfuscate_key(""), "");
        assert_eq!(deobfuscate_key(""), "");
    }
}
