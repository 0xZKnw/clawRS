//! UI components for LocaLM
//!
//! This module contains all user interface components built with Dioxus.

pub mod chat;
pub mod components;
pub mod settings;
pub mod sidebar;

use crate::ui::sidebar::Sidebar;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Theme {
    Light,
    Dark,
}

impl Theme {
    fn toggle(&self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

/// Main Application Layout
#[component]
pub fn Layout() -> Element {
    // Default to Dark theme
    let mut theme = use_signal(|| Theme::Dark);

    rsx! {
        // Theme wrapper
        div {
            "data-theme": "{theme().as_str()}",
            class: "flex h-screen w-screen bg-[var(--bg-main)] text-[var(--text-primary)] transition-colors duration-300 overflow-hidden font-sans",

            // Link CSS - In a real build step we might bundle this, but for dev this works
            link { rel: "stylesheet", href: "assets/styles.css" }

            // Sidebar
            Sidebar {}

            // Main Content Area
            main {
                class: "flex-1 flex flex-col h-full relative min-w-0 bg-[var(--bg-main)]",

                // Theme Toggle (Temporary location until Settings is built)
                div {
                    class: "absolute top-4 right-4 z-50",
                    button {
                        onclick: move |_| theme.set(theme().toggle()),
                        class: "p-2 rounded-full hover:bg-[var(--bg-hover)] text-[var(--text-tertiary)] hover:text-[var(--text-primary)] transition-all active:scale-95",
                        title: "Toggle Theme",

                        if theme() == Theme::Dark {
                            // Sun icon
                            svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round", circle { cx: "12", cy: "12", r: "5" }, path { d: "M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" } }
                        } else {
                            // Moon icon
                            svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round", path { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" } }
                        }
                    }
                }

                // Main Content Placeholder (Welcome Screen)
                div {
                    class: "flex-1 flex flex-col items-center justify-center p-8 text-center animate-fade-in",

                    div {
                        class: "mb-8 transform hover:scale-105 transition-transform duration-500",
                        div {
                            class: "w-24 h-24 bg-gradient-to-br from-[var(--accent-primary)] to-[var(--accent-hover)] rounded-3xl mx-auto shadow-xl flex items-center justify-center text-white mb-6",
                            svg { width: "48", height: "48", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5", path { d: "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" } }
                        }

                        h1 { class: "text-4xl font-bold mb-3 tracking-tight text-[var(--text-primary)]", "LocaLM" }
                        p { class: "text-lg text-[var(--text-secondary)] max-w-md mx-auto leading-relaxed", "Your private AI companion. Fast, local, and secure." }
                    }

                    // Quick Actions (Placeholder)
                    div {
                        class: "grid grid-cols-1 md:grid-cols-2 gap-4 w-full max-w-lg",

                        // Action 1
                        button {
                            class: "flex items-center gap-4 p-4 rounded-xl border border-[var(--border-subtle)] hover:border-[var(--border-focus)] hover:bg-[var(--bg-hover)] transition-all text-left group",
                            div {
                                class: "p-2 bg-[var(--bg-active)] rounded-lg text-[var(--accent-primary)] group-hover:scale-110 transition-transform",
                                svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", path { d: "M12 5v14M5 12h14" } }
                            }
                            div {
                                div { class: "font-medium text-[var(--text-primary)]", "New Conversation" }
                                div { class: "text-sm text-[var(--text-tertiary)]", "Start a fresh chat" }
                            }
                        }

                        // Action 2
                        button {
                            class: "flex items-center gap-4 p-4 rounded-xl border border-[var(--border-subtle)] hover:border-[var(--border-focus)] hover:bg-[var(--bg-hover)] transition-all text-left group",
                            div {
                                class: "p-2 bg-[var(--bg-active)] rounded-lg text-[var(--accent-primary)] group-hover:scale-110 transition-transform",
                                svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", circle { cx: "12", cy: "12", r: "3" }, path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 5 9.4a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" } }
                            }
                            div {
                                div { class: "font-medium text-[var(--text-primary)]", "Settings" }
                                div { class: "text-sm text-[var(--text-tertiary)]", "Configure models" }
                            }
                        }
                    }
                }
            }
        }
    }
}
