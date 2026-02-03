//! UI components for LocaLM
//!
//! This module contains all user interface components built with Dioxus.

pub mod chat;
pub mod components;
pub mod settings;
pub mod sidebar;

use crate::ui::sidebar::Sidebar;
use crate::ui::chat::ChatView;
use crate::ui::settings::Settings as SettingsPanel;
use crate::app::AppState;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum MainView {
    Chat,
    Settings,
}

/// Main Application Layout
#[component]
pub fn Layout() -> Element {
    let mut current_view = use_signal(|| MainView::Chat);
    let app_state = use_context::<AppState>();
    
    // Get theme from settings
    let theme_str = app_state.settings.read().theme.clone();

    rsx! {
        // Theme wrapper
        div {
            "data-theme": "{theme_str}",
            class: "flex h-screen w-screen bg-[var(--bg-main)] text-[var(--text-primary)] transition-colors duration-300 overflow-hidden",

            // Inline CSS using include_str! macro for desktop app compatibility
            style { {include_str!("../../assets/styles.css")} }

            // Sidebar
            Sidebar {
                on_settings_click: move |_| current_view.set(MainView::Settings),
                on_new_chat: move |_| current_view.set(MainView::Chat)
            }

            // Main Content Area
            main {
                class: "flex-1 flex flex-col h-full relative min-w-0 bg-[var(--bg-main)] transition-colors duration-300",

                // Main Content (ChatView, Settings, or Welcome Screen)
                if current_view() == MainView::Settings {
                    div {
                        class: "flex flex-col h-full",
                        // Back Button Header
                        div {
                            class: "flex-none px-8 pt-6 pb-2",
                            button {
                                onclick: move |_| current_view.set(MainView::Chat),
                                class: "flex items-center gap-2 text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors text-sm font-medium group",
                                svg {
                                    class: "w-4 h-4 transition-transform group-hover:-translate-x-1",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    path { d: "M19 12H5M12 19l-7-7 7-7" }
                                }
                                "Back to Chat"
                            }
                        }
                        SettingsPanel {}
                    }
                } else if app_state.current_conversation.read().is_some() {
                    ChatView {}
                } else {
                    // Welcome Screen - Modern & Minimalist
                    div {
                        class: "flex-1 flex flex-col items-center justify-center p-8 text-center animate-fade-in-up",

                        div {
                            class: "mb-10 relative group",
                            // Decorative blur behind logo
                            div { class: "absolute inset-0 bg-[var(--accent-primary)] blur-[60px] opacity-20 rounded-full group-hover:opacity-30 transition-all duration-500" }
                            
                            div {
                                class: "relative w-24 h-24 bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-2xl flex items-center justify-center text-[var(--accent-primary)] shadow-lg mb-6 transform group-hover:scale-105 transition-all duration-500",
                                svg { 
                                    width: "40", 
                                    height: "40", 
                                    view_box: "0 0 24 24", 
                                    fill: "none", 
                                    stroke: "currentColor", 
                                    stroke_width: "1.5", 
                                    path { d: "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" } 
                                }
                            }
                        }

                        h1 { 
                            class: "text-4xl font-bold mb-3 tracking-tight text-[var(--text-primary)]", 
                            "LocaLM"
                        }
                        p { 
                            class: "text-lg text-[var(--text-secondary)] max-w-md mx-auto leading-relaxed mb-12", 
                            "Your private AI companion. Fast, local, and secure." 
                        }

                        // Quick Actions
                        div {
                            class: "grid grid-cols-1 md:grid-cols-2 gap-4 w-full max-w-lg",

                            // Action 1: New Conversation
                            button {
                                class: "flex items-center gap-4 p-5 rounded-2xl bg-[var(--bg-surface)] border border-[var(--border-subtle)] hover:border-[var(--accent-primary)] hover:shadow-glow transition-all duration-300 group text-left",
                                div {
                                    class: "p-3 bg-[var(--bg-subtle)] rounded-xl text-[var(--accent-primary)] group-hover:bg-[var(--accent-primary)] group-hover:text-white transition-colors duration-300",
                                    svg { 
                                        width: "20", 
                                        height: "20", 
                                        view_box: "0 0 24 24", 
                                        fill: "none", 
                                        stroke: "currentColor", 
                                        stroke_width: "2", 
                                        path { d: "M12 5v14M5 12h14" } 
                                    }
                                }
                                div {
                                    div { class: "font-semibold text-[var(--text-primary)] mb-0.5", "New Conversation" }
                                    div { class: "text-sm text-[var(--text-tertiary)] group-hover:text-[var(--text-secondary)] transition-colors", "Start a fresh chat" }
                                }
                            }

                            // Action 2: Settings
                            button {
                                onclick: move |_| current_view.set(MainView::Settings),
                                class: "flex items-center gap-4 p-5 rounded-2xl bg-[var(--bg-surface)] border border-[var(--border-subtle)] hover:border-[var(--accent-primary)] hover:shadow-glow transition-all duration-300 group text-left",
                                div {
                                    class: "p-3 bg-[var(--bg-subtle)] rounded-xl text-[var(--text-secondary)] group-hover:bg-[var(--text-primary)] group-hover:text-[var(--bg-main)] transition-colors duration-300",
                                    svg { 
                                        width: "20", 
                                        height: "20", 
                                        view_box: "0 0 24 24", 
                                        fill: "none", 
                                        stroke: "currentColor", 
                                        stroke_width: "2", 
                                        circle { cx: "12", cy: "12", r: "3" }, 
                                        path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" } 
                                    }
                                }
                                div {
                                    div { class: "font-semibold text-[var(--text-primary)] mb-0.5", "Settings" }
                                    div { class: "text-sm text-[var(--text-tertiary)]", "Configure models" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
