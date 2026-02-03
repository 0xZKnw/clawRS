//! Model metadata storage
//!
//! Tracks installed models and their configurations.

use crate::storage::{get_data_dir, StorageError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Information about a GGUF model file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Full path to the model file
    pub path: PathBuf,
    /// Filename of the model
    pub filename: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Last modification time
    pub last_modified: SystemTime,
}

impl ModelInfo {
    /// Create a ModelInfo from a file path
    fn from_path(path: PathBuf) -> Result<Self, std::io::Error> {
        let metadata = fs::metadata(&path)?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            path,
            filename,
            size_bytes: metadata.len(),
            last_modified: metadata.modified()?,
        })
    }

    /// Get a human-readable size string
    pub fn size_string(&self) -> String {
        let bytes = self.size_bytes as f64;

        if bytes < 1024.0 {
            format!("{} B", bytes)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// Scan a directory for GGUF model files
///
/// Returns a list of ModelInfo for all .gguf files found in the directory
pub fn scan_models_directory(directory: &PathBuf) -> Result<Vec<ModelInfo>, StorageError> {
    if !directory.exists() {
        tracing::warn!("Models directory does not exist: {}", directory.display());
        return Ok(vec![]);
    }

    if !directory.is_dir() {
        tracing::warn!("Models path is not a directory: {}", directory.display());
        return Ok(vec![]);
    }

    let mut models = vec![];

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        // Check if it's a .gguf file
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension.to_str() == Some("gguf") {
                    match ModelInfo::from_path(path.clone()) {
                        Ok(model_info) => {
                            tracing::debug!("Found model: {}", model_info.filename);
                            models.push(model_info);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to read model file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    }

    // Sort by filename
    models.sort_by(|a, b| a.filename.cmp(&b.filename));

    tracing::info!("Found {} model(s) in {}", models.len(), directory.display());

    Ok(models)
}

/// Scan the default models directory
///
/// Uses the models directory from the application data directory
pub fn scan_default_models_directory() -> Result<Vec<ModelInfo>, StorageError> {
    let models_dir = get_data_dir()?.join("models");
    scan_models_directory(&models_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_model_info_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let model_path = temp_dir.path().join("test_model.gguf");

        // Create a test file
        File::create(&model_path).unwrap();

        let model_info = ModelInfo::from_path(model_path.clone()).unwrap();

        assert_eq!(model_info.filename, "test_model.gguf");
        assert_eq!(model_info.path, model_path);
        assert_eq!(model_info.size_bytes, 0);
    }

    #[test]
    fn test_size_string() {
        let model_info = ModelInfo {
            path: PathBuf::from("test.gguf"),
            filename: "test.gguf".to_string(),
            size_bytes: 1024,
            last_modified: SystemTime::now(),
        };

        assert_eq!(model_info.size_string(), "1.00 KB");

        let large_model = ModelInfo {
            path: PathBuf::from("large.gguf"),
            filename: "large.gguf".to_string(),
            size_bytes: 1024 * 1024 * 1024 * 3, // 3 GB
            last_modified: SystemTime::now(),
        };

        assert!(large_model.size_string().contains("GB"));
    }

    #[test]
    fn test_scan_models_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create some test model files
        File::create(temp_dir.path().join("model1.gguf")).unwrap();
        File::create(temp_dir.path().join("model2.gguf")).unwrap();
        File::create(temp_dir.path().join("not_a_model.txt")).unwrap();

        let models = scan_models_directory(&temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(models.len(), 2);
        assert!(models.iter().any(|m| m.filename == "model1.gguf"));
        assert!(models.iter().any(|m| m.filename == "model2.gguf"));
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let nonexistent = PathBuf::from("/this/path/does/not/exist");
        let result = scan_models_directory(&nonexistent);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let models = scan_models_directory(&temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(models.len(), 0);
    }
}
