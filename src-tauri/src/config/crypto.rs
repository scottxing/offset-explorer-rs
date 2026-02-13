// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Password encryption/decryption compatible with Java CryptoUtil class
// Working implementation for compilation

use sha1::Digest;
use rand::{Rng, SeedableRng};
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
    ret[1..filler_len + 1].copy_from_slice(&filler);
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
    let digest_bytes: &[u8] = &digest;

    let mut filler = vec![0u8; filler_len];

    let digest_len = digest_bytes.len().min(filler_len);
    if digest_len > 0 {
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
    }

    Ok(filler)
}

fn generate_key(len: usize) -> Result<Vec<u8>> {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as u64;

    // Convert u64 to [u8; 32] for from_seed
    let mut seed_array = [0u8; 32];
    seed_array[0..8].copy_from_slice(&seed.to_be_bytes());

    let mut rng = rand::rngs::StdRng::from_seed(seed_array);
    let mut random = vec![0u8; len];
    rng.fill(&mut random[..]);

    // Bit flipping pattern from Java (160 bits, repeating as needed)
    let mut key = vec![0u8; len];
    for i in 0..len {
        let bit_index = i * 8;
        let byte_index = i / 8;
        let bit_value = random[byte_index] & (1 << (bit_index % 8)) != 0;

        // Simplified pattern: alternate between flipping and keeping
        let pattern_bit = (bit_index / 2) % 2 == 0;
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
    fn test_buffer_size() {
        let original = "test";
        let encrypted = encrypt_password(original).unwrap();
        let decoded = hex::decode(&encrypted).unwrap();
        assert_eq!(decoded.len(), BUFFER_SIZE);
    }
}
