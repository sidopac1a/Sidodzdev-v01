//! ISO Validator
//! 
//! Validates ISO file integrity using MD5, SHA1, and SHA256 hash algorithms.

use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;
use anyhow::{Result, Context};
use tracing::{info, debug, error};

use crate::core::events::HashType;

/// Calculate file hash using the specified algorithm
pub async fn validate_iso_hash(path: &Path, hash_type: HashType) -> Result<String> {
    info!("Calculating {} hash for: {:?}", hash_type, path);

    let file = File::open(path)
        .context("Failed to open file for hash calculation")?;

    let hash = match hash_type {
        HashType::Md5 => calculate_md5(file).await?,
        HashType::Sha1 => calculate_sha1(file).await?,
        HashType::Sha256 => calculate_sha256(file).await?,
    };

    info!("{} hash calculated: {}", hash_type, hash);
    Ok(hash)
}

/// Calculate MD5 hash
async fn calculate_md5(file: File) -> Result<String> {
    use md5::{Md5, Digest};

    let mut hasher = Md5::new();
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192]; // 8KB buffer

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Calculate SHA1 hash
async fn calculate_sha1(file: File) -> Result<String> {
    use sha1::{Sha1, Digest};

    let mut hasher = Sha1::new();
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Calculate SHA256 hash
async fn calculate_sha256(file: File) -> Result<String> {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Calculate hash for a file (sync version for utilities)
pub fn calculate_file_hash_sync(path: &Path, hash_type: HashType) -> Result<String> {
    let file = File::open(path)
        .context("Failed to open file for hash calculation")?;

    match hash_type {
        HashType::Md5 => {
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
        HashType::Sha1 => {
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
        HashType::Sha256 => {
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
    }
}

/// Verify ISO against a known hash
pub async fn verify_iso_against_hash(
    path: &Path,
    expected_hash: &str,
    hash_type: HashType,
) -> Result<bool> {
    info!("Verifying ISO against expected {} hash", hash_type);

    let calculated_hash = validate_iso_hash(path, hash_type).await?;

    let matches = calculated_hash.eq_ignore_ascii_case(expected_hash);

    if matches {
        info!("Hash verification PASSED");
    } else {
        error!(
            "Hash verification FAILED: expected={}, got={}",
            expected_hash, calculated_hash
        );
    }

    Ok(matches)
}

/// Calculate all three hashes (MD5, SHA1, SHA256)
pub async fn calculate_all_hashes(path: &Path) -> Result<(String, String, String)> {
    debug!("Calculating all hash types for: {:?}", path);

    let md5 = validate_iso_hash(path, HashType::Md5).await?;
    let sha1 = validate_iso_hash(path, HashType::Sha1).await?;
    let sha256 = validate_iso_hash(path, HashType::Sha256).await?;

    Ok((md5, sha1, sha256))
}
