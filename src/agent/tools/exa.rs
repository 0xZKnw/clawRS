//! Exa search tool for web search capabilities
//!
//! Provides AI-powered search using the Exa API

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::agent::tools::{Tool, ToolResult, ToolError};

/// Exa search tool configuration
#[derive(Clone, Debug)]
pub struct ExaSearchConfig {
    pub api_key: String,
    pub base_url: String,
}

impl Default for ExaSearchConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("EXA_API_KEY").unwrap_or_default(),
            base_url: "https://api.exa.ai".to_string(),
        }
    }
}

/// Exa search tool
pub struct ExaSearchTool {
    config: ExaSearchConfig,
    client: reqwest::Client,
}

impl ExaSearchTool {
    pub fn new(config: ExaSearchConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
    
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("EXA_API_KEY").ok()?;
        if api_key.is_empty() {
            return None;
        }
        Some(Self::new(ExaSearchConfig {
            api_key,
            ..Default::default()
        }))
    }
}

#[async_trait]
impl Tool for ExaSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }
    
    fn description(&self) -> &str {
        "Search the web for information using AI-powered search"
    }
    
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "num_results": {
                    "type": "integer",
                    "description": "Number of results to return (1-10)",
                    "minimum": 1,
                    "maximum": 10,
                    "default": 5
                }
            },
            "required": ["query"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<ToolResult, ToolError> {
        let query = params["query"].as_str()
            .ok_or_else(|| ToolError::InvalidParameters("query is required".to_string()))?;
        
        let num_results = params["num_results"].as_u64()
            .map(|n| n.clamp(1, 10) as usize)
            .unwrap_or(5);
        
        if self.config.api_key.is_empty() {
            return Err(ToolError::ExecutionFailed(
                "EXA_API_KEY not configured. Set the environment variable to enable web search.".to_string()
            ));
        }
        
        let request_body = serde_json::json!({
            "query": query,
            "numResults": num_results,
            "contents": {
                "text": true,
                "highlights": true
            }
        });
        
        let response = self.client
            .post(format!("{}/search", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ToolError::ExecutionFailed(
                format!("Exa API error ({}): {}", status, error_text)
            ));
        }
        
        let search_response: ExaSearchResponse = response.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse response: {}", e)))?;
        
        let results: Vec<Value> = search_response.results.iter().map(|r| {
            serde_json::json!({
                "title": r.title,
                "url": r.url,
                "content": r.text,
                "highlights": r.highlights,
            })
        }).collect();
        
        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "query": query,
                "results": results,
                "total": results.len()
            }),
            message: format!("Found {} results for '{}'", results.len(), query),
        })
    }
}

/// Exa API response structure
#[derive(Debug, Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaResult>,
}

#[derive(Debug, Deserialize)]
struct ExaResult {
    title: String,
    url: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    highlights: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exa_tool_name() {
        let config = ExaSearchConfig {
            api_key: "test".to_string(),
            ..Default::default()
        };
        let tool = ExaSearchTool::new(config);
        assert_eq!(tool.name(), "web_search");
    }
    
    #[test]
    fn test_exa_from_env_without_key() {
        // Unset the env var if it exists
        std::env::remove_var("EXA_API_KEY");
        assert!(ExaSearchTool::from_env().is_none());
    }
}
