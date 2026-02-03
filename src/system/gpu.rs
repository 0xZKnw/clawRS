//! GPU detection and management
//!
//! Detects available GPUs and their capabilities for model acceleration.

/// GPU information
#[derive(Debug, Clone, Default)]
pub struct GpuInfo {
    pub name: String,
    pub vram_total_mb: u64,
    pub vram_used_mb: u64,
    pub is_available: bool,
}

/// Detect available GPU
///
/// For now, returns mock data. Real implementation would use
/// llama-cpp-2's list_llama_ggml_backend_devices() or similar.
pub fn detect_gpu() -> GpuInfo {
    // TODO: Integrate with llama-cpp-2 backend detection
    // For now, return unavailable GPU
    GpuInfo {
        name: "Unknown GPU".to_string(),
        vram_total_mb: 0,
        vram_used_mb: 0,
        is_available: false,
    }
}
