//! LLM inference engine
//!
//! This module handles all interaction with llama-cpp for model loading and inference.

pub mod engine;
pub mod model;
pub mod streaming;

// Re-export main types for convenience
pub use engine::{EngineError, GenerationParams, LlamaEngine, LoadedModelInfo};
pub use model::{validate_gguf, GgufMetadata, ModelError, GGUF_MAGIC};
pub use streaming::StreamToken;
