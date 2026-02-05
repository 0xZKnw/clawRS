//! Permission dialog UI component
//!
//! Displays permission requests and allows user approval/denial

use crate::agent::permissions::{PermissionLevel, PermissionManager, PermissionRequest};
use crate::app::AppState;
use dioxus::prelude::*;

/// Permission dialog component
#[component]
pub fn PermissionDialog() -> Element {
    let _app_state = use_context::<AppState>();
    let pending_requests = use_signal(Vec::<PermissionRequest>::new);

    // Monitor pending requests
    use_effect(move || {
        // This would be connected to the permission manager's signals
        // For now, we'll use a mock implementation
    });

    let requests = pending_requests.read();

    if requests.is_empty() {
        return rsx! { div {} };
    }

    let current_request = &requests[0];

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4",

            // Dialog
            div {
                class: "w-full max-w-lg bg-[var(--bg-secondary)] rounded-2xl border border-[var(--border-subtle)] shadow-2xl overflow-hidden",

                // Header
                div {
                    class: "p-6 border-b border-[var(--border-subtle)]",

                    div {
                        class: "flex items-center gap-3 mb-2",

                        div {
                            class: "w-10 h-10 rounded-full bg-yellow-500/20 flex items-center justify-center",
                            span { class: "text-xl", "⚠️" }
                        }

                        h2 {
                            class: "text-lg font-semibold text-[var(--text-primary)]",
                            "Permission Required"
                        }
                    }

                    p {
                        class: "text-sm text-[var(--text-secondary)]",
                        "The AI agent is requesting permission to perform an action."
                    }
                }

                // Content
                div {
                    class: "p-6 space-y-4",

                    // Tool info
                    div {
                        class: "p-4 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-subtle)]",

                        div {
                            class: "flex items-center justify-between mb-2",

                            span {
                                class: "text-sm font-medium text-[var(--text-primary)]",
                                "Tool"
                            }

                            span {
                                class: "text-sm text-[var(--accent-primary)]",
                                "{current_request.tool_name}"
                            }
                        }

                        div {
                            class: "flex items-center justify-between mb-2",

                            span {
                                class: "text-sm font-medium text-[var(--text-primary)]",
                                "Operation"
                            }

                            span {
                                class: "text-sm text-[var(--text-secondary)]",
                                "{current_request.operation}"
                            }
                        }

                        div {
                            class: "flex items-center justify-between",

                            span {
                                class: "text-sm font-medium text-[var(--text-primary)]",
                                "Permission Level"
                            }

                            PermissionLevelBadge { level: current_request.level }
                        }
                    }

                    // Target
                    div {
                        class: "p-4 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-subtle)]",

                        span {
                            class: "text-xs uppercase tracking-wider text-[var(--text-tertiary)]",
                            "Target"
                        }

                        p {
                            class: "mt-1 text-sm font-mono text-[var(--text-secondary)] break-all",
                            "{current_request.target}"
                        }
                    }

                    // Parameters
                    details {
                        class: "p-4 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-subtle)]",

                        summary {
                            class: "text-xs uppercase tracking-wider text-[var(--text-tertiary)] cursor-pointer",
                            "Parameters"
                        }

                        pre {
                            class: "mt-2 text-xs text-[var(--text-secondary)] overflow-x-auto",
                            "{serde_json::to_string_pretty(&current_request.params).unwrap_or_default()}"
                        }
                    }
                }

                // Footer
                div {
                    class: "p-6 border-t border-[var(--border-subtle)] flex gap-3",

                    button {
                        class: "flex-1 px-4 py-2.5 rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-secondary)] font-medium hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] transition-colors",
                        onclick: move |_| {
                            // Deny permission
                        },
                        "Deny"
                    }

                    button {
                        class: "flex-1 px-4 py-2.5 rounded-lg bg-[var(--accent-primary)] text-white font-medium hover:bg-[var(--accent-secondary)] transition-colors",
                        onclick: move |_| {
                            // Approve permission
                        },
                        "Approve"
                    }
                }
            }
        }
    }
}

/// Permission level badge component
#[component]
fn PermissionLevelBadge(level: PermissionLevel) -> Element {
    let (label, color) = match level {
        PermissionLevel::ReadOnly => ("Read Only", "text-green-400 bg-green-400/10"),
        PermissionLevel::ReadWrite => ("Read/Write", "text-yellow-400 bg-yellow-400/10"),
        PermissionLevel::ExecuteSafe => ("Execute (Safe)", "text-orange-400 bg-orange-400/10"),
        PermissionLevel::ExecuteUnsafe => ("Execute (Unsafe)", "text-red-400 bg-red-400/10"),
        PermissionLevel::Network => ("Network", "text-cyan-400 bg-cyan-400/10"),
    };

    rsx! {
        span {
            class: "px-2 py-1 rounded-md text-xs font-medium {color}",
            "{label}"
        }
    }
}
