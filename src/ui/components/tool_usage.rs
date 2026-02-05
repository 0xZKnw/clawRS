//! Tool usage UI components
//!
//! Displays tool execution status and results in the chat interface

use crate::agent::tools::ToolResult;
use dioxus::prelude::*;

/// Component to display a tool being executed
#[component]
pub fn ToolExecutionCard(tool_name: String, status: String) -> Element {
    let (icon, color) = match tool_name.as_str() {
        "web_search" => ("🔍", "text-cyan-400"),
        "file_read" => ("📄", "text-blue-400"),
        "file_list" => ("📁", "text-blue-400"),
        "command" => ("⚡", "text-yellow-400"),
        _ => ("🔧", "text-gray-400"),
    };

    rsx! {
        div {
            class: "my-2 p-3 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-subtle)] flex items-center gap-3",

            span { class: "text-lg", "{icon}" }

            div {
                class: "flex-1",

                div {
                    class: "flex items-center gap-2",
                    span {
                        class: "text-sm font-medium {color}",
                        "{tool_name}"
                    }
                    span {
                        class: "text-xs text-[var(--text-tertiary)]",
                        "{status}"
                    }
                }
            }

            // Loading spinner
            if status == "running" {
                div {
                    class: "animate-spin w-4 h-4 border-2 border-[var(--accent-primary)] border-t-transparent rounded-full"
                }
            }
        }
    }
}

/// Component to display tool execution result
#[component]
pub fn ToolResultCard(tool_name: String, result: ToolResult) -> Element {
    let (icon, color) = match tool_name.as_str() {
        "web_search" => ("🔍", "text-cyan-400"),
        "file_read" => ("📄", "text-blue-400"),
        "file_list" => ("📁", "text-blue-400"),
        "command" => ("⚡", "text-yellow-400"),
        _ => ("🔧", "text-gray-400"),
    };

    let border_color = if result.success {
        "border-green-500/30"
    } else {
        "border-red-500/30"
    };

    let data_str = serde_json::to_string_pretty(&result.data)
        .unwrap_or_else(|_| "Error formatting data".to_string());

    rsx! {
        div {
            class: "my-2 rounded-lg bg-[var(--bg-tertiary)] border {border_color} overflow-hidden",

            // Header
            div {
                class: "p-3 flex items-center gap-3 border-b border-[var(--border-subtle)]",

                span { class: "text-lg", "{icon}" }

                span {
                    class: "text-sm font-medium {color}",
                    "{tool_name}"
                }

                if result.success {
                    span {
                        class: "ml-auto text-xs text-green-400",
                        "✓ Success"
                    }
                } else {
                    span {
                        class: "ml-auto text-xs text-red-400",
                        "✗ Failed"
                    }
                }
            }

            // Message
            div {
                class: "p-3 text-sm text-[var(--text-secondary)]",
                "{result.message}"
            }

            // Data preview (collapsed by default)
            details {
                class: "border-t border-[var(--border-subtle)]",

                summary {
                    class: "p-3 text-xs text-[var(--text-tertiary)] cursor-pointer hover:text-[var(--text-secondary)] transition-colors",
                    "View raw data"
                }

                pre {
                    class: "p-3 text-xs text-[var(--text-tertiary)] bg-[var(--bg-primary)] overflow-x-auto",
                    "{data_str}"
                }
            }
        }
    }
}

/// Component to display web search results
#[component]
pub fn WebSearchResults(query: String, results: Vec<serde_json::Value>) -> Element {
    let results_count = results.len();

    rsx! {
        div {
            class: "my-2 rounded-lg bg-[var(--bg-tertiary)] border border-cyan-500/30 overflow-hidden",

            // Header
            div {
                class: "p-3 flex items-center gap-3 border-b border-[var(--border-subtle)] bg-cyan-500/10",

                span { class: "text-lg", "🔍" }

                div {
                    class: "flex-1",

                    span {
                        class: "text-sm font-medium text-cyan-400",
                        "Web Search"
                    }

                    p {
                        class: "text-xs text-[var(--text-tertiary)]",
                        "{query}"
                    }
                }

                span {
                    class: "text-xs text-cyan-400",
                    "{results_count} results"
                }
            }

            // Results
            div {
                class: "max-h-96 overflow-y-auto",

                for result in results.iter() {
                    SearchResultItem { result: result.clone() }
                }
            }
        }
    }
}

#[component]
fn SearchResultItem(result: serde_json::Value) -> Element {
    let url = result.get("url").and_then(|v| v.as_str()).unwrap_or("#");
    let title = result
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled");
    let url_display = result.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let content = result.get("content").and_then(|v| v.as_str());

    rsx! {
        div {
            class: "p-3 border-b border-[var(--border-subtle)] last:border-b-0",

            a {
                href: "{url}",
                target: "_blank",
                class: "text-sm font-medium text-cyan-400 hover:underline block mb-1",
                "{title}"
            }

            p {
                class: "text-xs text-[var(--text-tertiary)] mb-2",
                "{url_display}"
            }

            if let Some(text) = content {
                p {
                    class: "text-sm text-[var(--text-secondary)] line-clamp-3",
                    "{text}"
                }
            }
        }
    }
}
