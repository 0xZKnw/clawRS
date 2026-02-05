//! Settings storage
//!
//! Manages persistence of user preferences and application settings.

use crate::storage::{get_data_dir, StorageError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Temperature parameter for text generation (0.0 - 2.0)
    pub temperature: f32,
    /// Top-p (nucleus sampling) parameter (0.0 - 1.0)
    pub top_p: f32,
    /// Top-k sampling parameter
    pub top_k: u32,
    /// Maximum number of tokens to generate
    pub max_tokens: u32,
    /// Context window size
    pub context_size: u32,
    /// System prompt prepended to conversations
    pub system_prompt: String,
    /// Number of GPU layers to offload (0 = CPU only)
    pub gpu_layers: u32,
    /// Directory where model files (.gguf) are stored
    pub models_directory: PathBuf,
    /// UI theme: "dark" or "light"
    pub theme: String,
    /// Font size: "small", "medium", or "large"
    pub font_size: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            max_tokens: 65536,    // 64k max output tokens
            context_size: 131072, // 128k context window
            system_prompt: r#"You are LocaLM, an AI assistant with access to tools and system capabilities.

## Your Capabilities

You have access to the following tools:

### 1. Web Search (web_search)
Search the internet for real-time information.
- Use this when you need current information, facts, or data
- Parameter: {"query": "your search query", "num_results": 5}

### 2. File System Access
- **file_read**: Read text files on the system
  - Use this to read code, documents, logs, or any text file
  - Parameter: {"path": "/absolute/path/to/file"}
  
- **file_list**: List directory contents
  - Use this to explore folders and see what files are available
  - Parameter: {"path": "/absolute/path/to/directory"}

### 3. Command Execution (command)
Execute safe shell commands.
- Allowed commands: ls, cat, echo, pwd, whoami, date, wc, head, tail, find, grep
- Use for: checking system info, reading logs, exploring directories
- Parameter: {"command": "ls -la", "timeout_secs": 30}

## When to Use Tools

**Use web_search when:**
- The user asks about current events, recent news, or time-sensitive information
- You need to verify facts or find specific data
- The query requires information beyond your training data

**Use file_read/file_list when:**
- The user asks about files on their system
- You need to analyze code, logs, or documents
- The user wants help with local files

**Use command when:**
- You need system information (date, current directory, etc.)
- The user asks to list files or search within files
- You need to check system status

## Tool Usage Format

When you need to use a tool, respond with a JSON object:
```json
{
  "tool": "tool_name",
  "params": {
    "param1": "value1",
    "param2": "value2"
  }
}
```

After the tool executes, you will receive the result and can provide your final response based on that information.

## Guidelines

1. **Always use tools** when the user's request requires information you don't have
2. **Ask for permission** before accessing sensitive files or executing commands outside the safe list
3. **Be concise** but thorough in your responses
4. **Explain your reasoning** when using tools - tell the user what you're doing and why
5. **Handle errors gracefully** - if a tool fails, explain what went wrong and suggest alternatives

## Examples

User: "What's the weather like in Paris?"
→ Use web_search: {"query": "current weather Paris France"}

User: "Can you read my config file?"
→ Ask: "What's the path to your config file?"

User: "Show me the files in my home directory"
→ Use command: {"command": "ls -la ~"}

User: "Help me debug this error"
→ Ask for the error log file path, then use file_read

Remember: You have a 128k context window and can generate up to 64k tokens, so you can handle long conversations and detailed responses."#.to_string(),
            gpu_layers: 99, // Offload all layers to GPU by default
            models_directory: get_data_dir()
                .ok()
                .map(|d| d.join("models"))
                .unwrap_or_else(|| PathBuf::from("./models")),
            theme: "dark".to_string(),
            font_size: "medium".to_string(),
        }
    }
}

impl AppSettings {
    /// Validate settings values
    ///
    /// Ensures all parameters are within acceptable ranges
    pub fn validate(&mut self) {
        // Clamp temperature between 0.0 and 2.0
        self.temperature = self.temperature.clamp(0.0, 2.0);

        // Clamp top_p between 0.0 and 1.0
        self.top_p = self.top_p.clamp(0.0, 1.0);

        // Ensure reasonable values for other parameters
        if self.top_k == 0 {
            self.top_k = 40;
        }

        // Clamp max_tokens between 1 and 65536 (64k)
        self.max_tokens = self.max_tokens.clamp(1, 65536);

        // Valid context sizes: 2048, 4096, 8192, 16384, 32768, 65536, 131072
        let valid_context_sizes = [2048, 4096, 8192, 16384, 32768, 65536, 131072];
        if !valid_context_sizes.contains(&self.context_size) {
            // Find closest valid size
            self.context_size = *valid_context_sizes
                .iter()
                .min_by_key(|&&size| (size as i64 - self.context_size as i64).abs())
                .unwrap_or(&131072);
        }

        // Validate theme
        if self.theme != "dark" && self.theme != "light" {
            self.theme = "dark".to_string();
        }

        // Validate font size
        if !["small", "medium", "large"].contains(&self.font_size.as_str()) {
            self.font_size = "medium".to_string();
        }
    }
}

/// Get the settings file path
fn get_settings_path() -> Result<PathBuf, StorageError> {
    Ok(get_data_dir()?.join("settings.json"))
}

/// Load settings from disk
///
/// Returns default settings if the file doesn't exist or is corrupted
pub fn load_settings() -> AppSettings {
    match load_settings_internal() {
        Ok(settings) => settings,
        Err(e) => {
            tracing::warn!("Failed to load settings, using defaults: {}", e);
            AppSettings::default()
        }
    }
}

/// Internal settings loading with error propagation
fn load_settings_internal() -> Result<AppSettings, StorageError> {
    let path = get_settings_path()?;

    if !path.exists() {
        tracing::info!("Settings file not found, using defaults");
        return Ok(AppSettings::default());
    }

    let json = fs::read_to_string(&path)?;
    let mut settings: AppSettings = serde_json::from_str(&json)?;

    // Validate loaded settings
    settings.validate();

    tracing::debug!("Loaded settings from disk");
    Ok(settings)
}

/// Save settings to disk
pub fn save_settings(settings: &AppSettings) -> Result<(), StorageError> {
    let path = get_settings_path()?;

    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(settings)?;
    fs::write(path, json)?;

    tracing::debug!("Saved settings to disk");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.temperature, 0.7);
        assert_eq!(settings.top_p, 0.9);
        assert_eq!(settings.top_k, 40);
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.font_size, "medium");
    }

    #[test]
    fn test_settings_validation() {
        let mut settings = AppSettings::default();

        // Test temperature clamping
        settings.temperature = 5.0;
        settings.validate();
        assert_eq!(settings.temperature, 2.0);

        settings.temperature = -1.0;
        settings.validate();
        assert_eq!(settings.temperature, 0.0);

        // Test top_p clamping
        settings.top_p = 2.0;
        settings.validate();
        assert_eq!(settings.top_p, 1.0);

        // Test invalid theme
        settings.theme = "invalid".to_string();
        settings.validate();
        assert_eq!(settings.theme, "dark");

        // Test invalid font size
        settings.font_size = "huge".to_string();
        settings.validate();
        assert_eq!(settings.font_size, "medium");
    }

    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings::default();

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.temperature, deserialized.temperature);
        assert_eq!(settings.top_p, deserialized.top_p);
        assert_eq!(settings.theme, deserialized.theme);
    }

    #[test]
    fn test_settings_persistence() {
        // Test that settings can be saved and loaded
        let settings = AppSettings::default();

        // Serialize and deserialize
        let json = serde_json::to_string_pretty(&settings).unwrap();
        let mut loaded: AppSettings = serde_json::from_str(&json).unwrap();
        loaded.validate();

        assert_eq!(settings.temperature, loaded.temperature);
        assert_eq!(settings.theme, loaded.theme);
    }
}
