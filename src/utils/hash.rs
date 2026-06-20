//! Hash Utilities
//! 
//! Synchronous hash calculation for file verification.

use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;
use anyhow::{Result, Context};
use tracing::{info, debug};

use crate::core::events::HashType;

/// Calculate file hash synchronously
pub fn calculate_file_hash(path: &Path, hash_type: HashType) -> Result<String> {
    debug!("Calculating {:?} hash for: {:?}", hash_type, path);

    let file = File::open(path)
        .context("Failed to open file for hash calculation")?;

    let hash = match hash_type {
        HashType::Md5 => calculate_md5(file)?,
        HashType::Sha1 => calculate_sha1(file)?,
        HashType::Sha256 => calculate_sha256(file)?,
    };

    info!("Hash calculated: {}", hash);
    Ok(hash)
}

/// Calculate MD5 hash
fn calculate_md5(file: File) -> Result<String> {
    use md5::{Md5, Digest};

    let mut hasher = Md5::new();
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Calculate SHA1 hash
fn calculate_sha1(file: File) -> Result<String> {
    use sha1::{Sha1, Digest};

    let mut hasher = Sha1::new();
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Calculate SHA256 hash
fn calculate_sha256(file: File) -> Result<String> {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Compare two hashes (case-insensitive)
pub fn compare_hashes(hash1: &str, hash2: &str) -> bool {
    hash1.eq_ignore_ascii_case(hash2)
}

/// Format hash for display (truncate if too long)
pub fn format_hash(hash: &str, max_length: usize) -> String {
    if hash.len() > max_length {
        format!("{}...{}", &hash[..max_length/2], &hash[hash.len()-max_length/2..])
    } else {
        hash.to_string()
    }
}
