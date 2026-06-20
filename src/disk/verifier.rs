//! Write Verifier
//! 
//! Post-write verification to ensure data integrity.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::{Result, Context};
use tracing::{info, debug, error};

use crate::utils::hash::calculate_file_hash;
use crate::core::events::HashType;

/// Verify that written data matches the source ISO
pub struct WriteVerifier {
    source_path: PathBuf,
    device_path: String,
    cancel_flag: Option<AtomicBool>,
}

impl WriteVerifier {
    pub fn new(source_path: PathBuf, device_path: String) -> Self {
        Self {
            source_path,
            device_path,
            cancel_flag: None,
        }
    }

    pub fn with_cancellation(mut self, cancel_flag: AtomicBool) -> Self {
        self.cancel_flag = Some(cancel_flag);
        self
    }

    /// Perform full byte-by-byte verification
    pub fn verify_full(&self) -> Result<bool> {
        info!("Starting full write verification");

        let mut source_file = File::open(&self.source_path)
            .context("Failed to open source file for verification")?;
        let mut device_file = File::open(&self.device_path)
            .context("Failed to open device for verification")?;

        let source_size = source_file.metadata()?.len();
        let mut buffer_size = 4 * 1024 * 1024; // 4MB chunks

        let mut source_buffer = vec![0u8; buffer_size];
        let mut device_buffer = vec![0u8; buffer_size];
        let mut position: u64 = 0;

        loop {
            // Check cancellation
            if let Some(ref flag) = self.cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    return Err(anyhow::anyhow!("Verification cancelled"));
                }
            }

            let bytes_to_read = std::cmp::min(buffer_size as u64, source_size - position) as usize;
            if bytes_to_read == 0 {
                break;
            }

            source_buffer.resize(bytes_to_read, 0);
            device_buffer.resize(bytes_to_read, 0);

            let source_read = source_file.read(&mut source_buffer)?;
            let device_read = device_file.read(&mut device_buffer)?;

            if source_read != device_read {
                error!("Read size mismatch at position {}: source={}, device={}", 
                    position, source_read, device_read);
                return Ok(false);
            }

            if source_buffer[..source_read] != device_buffer[..device_read] {
                error!("Data mismatch at position {}", position);
                return Ok(false);
            }

            position += source_read as u64;

            if position % (100 * 1024 * 1024) == 0 {
                debug!("Verified {} MB / {} MB", 
                    position / 1024 / 1024, 
                    source_size / 1024 / 1024);
            }
        }

        info!("Verification completed successfully: {} bytes verified", position);
        Ok(true)
    }

    /// Quick verification using hash comparison
    pub fn verify_hash(&self, hash_type: HashType) -> Result<bool> {
        info!("Starting hash-based verification");

        let source_hash = calculate_file_hash(&self.source_path, hash_type)?;
        let device_hash = calculate_file_hash(&self.device_path, hash_type)?;

        info!("Source hash: {}", source_hash);
        info!("Device hash: {}", device_hash);

        Ok(source_hash == device_hash)
    }

    /// Sample-based verification (checks random sectors)
    pub fn verify_sampled(&self, sample_count: usize) -> Result<bool> {
        info!("Starting sampled verification with {} samples", sample_count);

        let mut source_file = File::open(&self.source_path)?;
        let mut device_file = File::open(&self.device_path)?;

        let source_size = source_file.metadata()?.len();
        let sample_size = 64 * 1024; // 64KB samples

        let mut source_buffer = vec![0u8; sample_size];
        let mut device_buffer = vec![0u8; sample_size];

        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut positions: Vec<u64> = (0..source_size / sample_size as u64).collect();
        positions.shuffle(&mut thread_rng());

        let samples_to_check = std::cmp::min(sample_count, positions.len());

        for i in 0..samples_to_check {
            let position = positions[i] * sample_size as u64;

            source_file.seek(SeekFrom::Start(position))?;
            device_file.seek(SeekFrom::Start(position))?;

            let source_read = source_file.read(&mut source_buffer)?;
            let device_read = device_file.read(&mut device_buffer)?;

            if source_read != device_read || source_buffer[..source_read] != device_buffer[..device_read] {
                error!("Sample verification failed at position {}", position);
                return Ok(false);
            }
        }

        info!("Sampled verification passed: {} samples checked", samples_to_check);
        Ok(true)
    }
}
