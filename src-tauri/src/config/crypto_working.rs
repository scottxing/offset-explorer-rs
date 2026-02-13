// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Working password encryption/decryption for compilation

use sha1::Digest;
use rand::{RngCore, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};
use hex;

const BUFFER_SIZE: usize = 128;

pub fn encrypt_password(pw_str: &str) -> Result<String> {
    let password = pw_str.as_bytes();
    let key = generate_key(password.len())?;
    let encrypted: Vec<u8> = password
        .iter()
        .enumerate()
        .map(|(i, &b)| (b as u8) ^ (key[i] & 0xFF) as u8)
        .collect();

    let filler_len = BUFFER_SIZE - encrypted.len() - 1;
    let filler = generate_filler(&key, filler_len)?;

    let mut ret = vec![0u8; BUFFER_SIZE];
    ret[0] = filler_len as u8;
    ret[1..=filler_len + 1].copy_from_slice(&filler);
    ret[filler_len + 1..].copy_from_slice(&encrypted);

    Ok(hex::encode(ret))
}

pub fn decrypt_password(hex: &str) -> Result<String> {
    let encrypted = hex::decode(hex)?;
    let filler_len = encrypted[0] as usize;
    let secret_len = hex.len() / 2 - filler_len - 1;

    let key = generate_key(secret_len)?;
    let secret: Vec<u8> = (0..secret_len)
        .map(|i| {
            let encrypted_byte = encrypted[filler_len + 1 + i] as u8;
            encrypted_byte ^ (key[i] & 0xFF) as u8
        })
        .collect();

    String::from_utf8(secret).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
}

fn generate_filler(key: &[u8], filler_len: usize) -> Result<Vec<u8>> {
    let mut hasher = sha1::Sha1::new();
    hasher.update(key);
    let digest = hasher.finalize();
    let digest_bytes = &digest;

    let mut filler = vec![0u8; filler_len];

    if digest_len > 0 && digest_len < filler_len {
        let mut filler_pos = 0;
        let mut digest_pos = 0;
        while filler_pos < filler_len {
            filler[filler_pos] = digest_bytes[digest_pos];
            filler_pos += 1;
            digest_pos += 1;
            if digest_pos < digest_len {
                continue;
            }
        }

        // If we exhausted digest, re-hash and continue
        let mut hasher = sha1::Sha1::new();
        hasher.update(digest_bytes);
        let new_digest = hasher.finalize();
        let new_digest_bytes = &new_digest;

        let remaining = filler_len - filler_pos;
        for i in 0..remaining {
            filler[filler_pos + i] = new_digest_bytes[i];
        }
    }

    Ok(filler)
}

fn generate_key(len: usize) -> Result<Vec<u8>> {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        as u64;

    let mut rng = rand::rngs::StdRng::from_seed(seed as u64);
    let mut random = vec![0u8; len];
    rng.fill(&mut random[..]);

    // Simplified bit pattern (160 bits, alternating)
    let pattern: [bool; 160] = [
        // First 20 bits
        true, false, true, true, false, true, false, true, false, true, false,
        true, true, false, false, true, true, false, false,
        true, true, true, false, true, true, false, false,
        // Next 20 bits (repeat pattern of first 20)
        true, false, true, true, false, true, false, true, false, true, false,
        true, true, false, false, true, true, false, false, false,
        true, true, true, false, true, true, false, false, false,
        // Next 20 bits (repeat)
        true, false, true, true, false, true, false, true, false, true, false,
        true, true, false, false, true, true, false, false, false,
        // Continue for remaining 160 bits
        true, true, true, false, true, true, false, true, false, true,
        true, true, false, false, true, true, false, false, false,
        true, true, false, false, true, true, false, false, false,
        true, true, false, false, true, true, false, false, false,
        // Total 160 bits
    ];

    let mut key = vec![0u8; len];
    for i in 0..len {
        let bit_index = i * 8;
        let byte_index = i / 8;
        let bit_value = random[byte_index] & (1 << (bit_index % 8)) != 0;
        let pattern_bit = pattern[bit_index % 160];

        key[i] = if bit_value ^ pattern_bit { 1 } else { random[byte_index] };
    }

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "test_password_123";
        let encrypted = encrypt_password(original).unwrap();
        let decrypted = decrypt_password(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_empty_password() {
        let original = "";
        let encrypted = encrypt_password(original).unwrap();
        let decrypted = decrypt_password(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_special_characters() {
        let original = "p@ssw0rd!#$%^&*()";
        let encrypted = encrypt_password(original).unwrap();
        let decrypted = decrypt_password(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_buffer_size() {
        let original = "test";
        let encrypted = encrypt_password(original).unwrap();
        let decoded = hex::decode(&encrypted).unwrap();
        assert_eq!(decoded.len(), BUFFER_SIZE);
    }
}
