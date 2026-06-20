//! Async Utilities
//! 
//! Helper functions for async operations and file I/O.

use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use tokio::task;
use anyhow::{Result, Context};
use tracing::{info, debug};

/// Async file reader for large files
pub struct AsyncFileReader {
    file_path: std::path::PathBuf,
    buffer_size: usize,
}

impl AsyncFileReader {
    pub fn new(path: impl AsRef<Path>, buffer_size: usize) -> Self {
        Self {
            file_path: path.as_ref().to_path_buf(),
            buffer_size,
        }
    }

    /// Read file in chunks asynchronously
    pub async fn read_chunks<F>(&self, mut callback: F) -> Result<u64>
    where
        F: FnMut(&[u8]) -> Result<bool> + Send + 'static,
    {
        let path = self.file_path.clone();
        let buffer_size = self.buffer_size;

        task::spawn_blocking(move || {
            let mut file = File::open(&path)
                .context("Failed to open file for reading")?;

            let mut buffer = vec![0u8; buffer_size];
            let mut total_read: u64 = 0;

            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }

                total_read += bytes_read as u64;

                if !callback(&buffer[..bytes_read])? {
                    break;
                }
            }

            Ok(total_read)
        })
        .await
        .context("Async read task failed")?
    }

    /// Get file size
    pub async fn file_size(&self) -> Result<u64> {
        let path = self.file_path.clone();

        task::spawn_blocking(move || {
            let metadata = std::fs::metadata(&path)?;
            Ok(metadata.len())
        })
        .await
        .context("Async file size task failed")?
    }
}

/// Async file writer for large files
pub struct AsyncFileWriter {
    file_path: std::path::PathBuf,
    buffer_size: usize,
}

impl AsyncFileWriter {
    pub fn new(path: impl AsRef<Path>, buffer_size: usize) -> Self {
        Self {
            file_path: path.as_ref().to_path_buf(),
            buffer_size,
        }
    }

    /// Write data in chunks asynchronously
    pub async fn write_chunks<F>(&self, mut data_source: F) -> Result<u64>
    where
        F: FnMut(&mut [u8]) -> Result<Option<usize>> + Send + 'static,
    {
        let path = self.file_path.clone();
        let buffer_size = self.buffer_size;

        task::spawn_blocking(move || {
            let mut file = File::create(&path)
                .context("Failed to create file for writing")?;

            let mut buffer = vec![0u8; buffer_size];
            let mut total_written: u64 = 0;

            loop {
                match data_source(&mut buffer)? {
                    Some(bytes_to_write) => {
                        file.write_all(&buffer[..bytes_to_write])?;
                        total_written += bytes_to_write as u64;
                    }
                    None => break,
                }
            }

            file.sync_all()?;
            Ok(total_written)
        })
        .await
        .context("Async write task failed")?
    }
}

/// Copy file with progress callback
pub async fn copy_file_with_progress(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    mut progress_callback: impl FnMut(u64, u64) + Send + 'static,
) -> Result<u64> {
    let source = source.as_ref().to_path_buf();
    let destination = destination.as_ref().to_path_buf();

    task::spawn_blocking(move || {
        let mut source_file = File::open(&source)
            .context("Failed to open source file")?;
        let mut dest_file = File::create(&destination)
            .context("Failed to create destination file")?;

        let total_size = source_file.metadata()?.len();
        let mut buffer = vec![0u8; 4 * 1024 * 1024]; // 4MB buffer
        let mut total_written: u64 = 0;

        loop {
            let bytes_read = source_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            dest_file.write_all(&buffer[..bytes_read])?;
            total_written += bytes_read as u64;

            progress_callback(total_written, total_size);
        }

        dest_file.sync_all()?;
        Ok(total_written)
    })
    .await
    .context("Async copy task failed")?
}

/// Sleep for a specified duration (async wrapper)
pub async fn async_sleep_ms(millis: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(millis)).await;
}

/// Run a blocking operation in a thread pool
pub async fn run_blocking<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    task::spawn_blocking(f)
        .await
        .context("Blocking task failed")?
}

/// Check if a file is accessible (not locked)
pub async fn is_file_accessible(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref().to_path_buf();

    task::spawn_blocking(move || {
        File::open(&path).is_ok()
    })
    .await
    .unwrap_or(false)
}
