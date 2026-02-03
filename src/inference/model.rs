//! Model management
//!
//! Handles model loading, unloading, and configuration.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use thiserror::Error;

/// GGUF magic bytes (little-endian: "GGUF")
pub const GGUF_MAGIC: u32 = 0x46554747;

/// Errors that can occur during model operations
#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Failed to open file: {0}")]
    FileOpen(#[from] std::io::Error),

    #[error("Invalid GGUF file: magic bytes mismatch (expected 0x{:08X}, got 0x{:08X})", GGUF_MAGIC, .0)]
    InvalidMagic(u32),

    #[error("Unsupported GGUF version: {0}")]
    UnsupportedVersion(u32),

    #[error("File too small to be valid GGUF")]
    FileTooSmall,
}

/// Metadata extracted from a GGUF file header
#[derive(Debug, Clone)]
pub struct GgufMetadata {
    /// GGUF format version
    pub version: u32,
    /// Number of tensors in the model
    pub tensor_count: u64,
    /// Number of metadata key-value pairs
    pub metadata_kv_count: u64,
}

/// Validates that a file is a valid GGUF format and extracts basic metadata.
///
/// # Arguments
/// * `path` - Path to the GGUF file
///
/// # Returns
/// * `Ok(GgufMetadata)` - Metadata extracted from valid GGUF file
/// * `Err(ModelError)` - If the file is not a valid GGUF file
pub fn validate_gguf<P: AsRef<Path>>(path: P) -> Result<GgufMetadata, ModelError> {
    let mut file = File::open(path)?;

    // Check file size (minimum: magic(4) + version(4) + tensor_count(8) + metadata_kv_count(8) = 24 bytes)
    let file_size = file.seek(SeekFrom::End(0))?;
    if file_size < 24 {
        return Err(ModelError::FileTooSmall);
    }
    file.seek(SeekFrom::Start(0))?;

    // Read magic bytes (4 bytes, little-endian)
    let mut magic_bytes = [0u8; 4];
    file.read_exact(&mut magic_bytes)?;
    let magic = u32::from_le_bytes(magic_bytes);

    if magic != GGUF_MAGIC {
        return Err(ModelError::InvalidMagic(magic));
    }

    // Read version (4 bytes, little-endian)
    let mut version_bytes = [0u8; 4];
    file.read_exact(&mut version_bytes)?;
    let version = u32::from_le_bytes(version_bytes);

    // GGUF v2 and v3 are supported
    if version < 2 || version > 3 {
        return Err(ModelError::UnsupportedVersion(version));
    }

    // Read tensor count (8 bytes, little-endian)
    let mut tensor_count_bytes = [0u8; 8];
    file.read_exact(&mut tensor_count_bytes)?;
    let tensor_count = u64::from_le_bytes(tensor_count_bytes);

    // Read metadata kv count (8 bytes, little-endian)
    let mut metadata_kv_count_bytes = [0u8; 8];
    file.read_exact(&mut metadata_kv_count_bytes)?;
    let metadata_kv_count = u64::from_le_bytes(metadata_kv_count_bytes);

    Ok(GgufMetadata {
        version,
        tensor_count,
        metadata_kv_count,
    })
}

/// Checks if a file appears to be a GGUF model file based on extension and magic bytes.
pub fn is_gguf_file<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    // Check extension first (quick check)
    if let Some(ext) = path.extension() {
        if ext.to_string_lossy().to_lowercase() != "gguf" {
            return false;
        }
    } else {
        return false;
    }

    // Validate magic bytes
    validate_gguf(path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_gguf() -> NamedTempFile {
        let mut file = tempfile::Builder::new().suffix(".gguf").tempfile().unwrap();

        // Write valid GGUF header
        file.write_all(&GGUF_MAGIC.to_le_bytes()).unwrap(); // magic
        file.write_all(&3u32.to_le_bytes()).unwrap(); // version 3
        file.write_all(&10u64.to_le_bytes()).unwrap(); // tensor_count
        file.write_all(&5u64.to_le_bytes()).unwrap(); // metadata_kv_count
        file.flush().unwrap();

        file
    }

    #[test]
    fn test_validate_gguf_valid() {
        let file = create_test_gguf();
        let metadata = validate_gguf(file.path()).unwrap();

        assert_eq!(metadata.version, 3);
        assert_eq!(metadata.tensor_count, 10);
        assert_eq!(metadata.metadata_kv_count, 5);
    }

    #[test]
    fn test_validate_gguf_invalid_magic() {
        let mut file = tempfile::Builder::new().suffix(".gguf").tempfile().unwrap();

        // Write invalid magic
        file.write_all(&0xDEADBEEFu32.to_le_bytes()).unwrap();
        file.write_all(&3u32.to_le_bytes()).unwrap();
        file.write_all(&10u64.to_le_bytes()).unwrap();
        file.write_all(&5u64.to_le_bytes()).unwrap();
        file.flush().unwrap();

        let result = validate_gguf(file.path());
        assert!(matches!(result, Err(ModelError::InvalidMagic(0xDEADBEEF))));
    }

    #[test]
    fn test_validate_gguf_file_too_small() {
        let mut file = tempfile::Builder::new().suffix(".gguf").tempfile().unwrap();

        // Write only magic bytes
        file.write_all(&GGUF_MAGIC.to_le_bytes()).unwrap();
        file.flush().unwrap();

        let result = validate_gguf(file.path());
        assert!(matches!(result, Err(ModelError::FileTooSmall)));
    }

    #[test]
    fn test_is_gguf_file() {
        let file = create_test_gguf();
        assert!(is_gguf_file(file.path()));
    }

    #[test]
    fn test_is_gguf_file_wrong_extension() {
        let mut file = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();

        // Write valid GGUF content but wrong extension
        file.write_all(&GGUF_MAGIC.to_le_bytes()).unwrap();
        file.write_all(&3u32.to_le_bytes()).unwrap();
        file.write_all(&10u64.to_le_bytes()).unwrap();
        file.write_all(&5u64.to_le_bytes()).unwrap();
        file.flush().unwrap();

        assert!(!is_gguf_file(file.path()));
    }
}
