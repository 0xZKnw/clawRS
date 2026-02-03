use crate::app::AppState;
use crate::storage::settings::save_settings;
use dioxus::prelude::*;

pub fn AppearanceSettings() -> Element {
    let app_state = use_context::<AppState>();
    let settings = app_state.settings.read().clone();
    let dark_mode = settings.theme == "dark";
    let font_size = settings.font_size.to_lowercase();
    let selected_font_size = match font_size.as_str() {
        "small" => "Small",
        "large" => "Large",
        _ => "Medium",
    };
    let mut app_state_theme = app_state.clone();
    let mut app_state_font_size = app_state.clone();

    rsx! {
        div {
            class: "space-y-8 max-w-3xl mx-auto animate-fade-in",
            style: "padding-bottom: 2rem;",

            h3 {
                class: "text-xl font-semibold pb-2",
                style: "color: var(--text-primary); border-bottom: 1px solid var(--border-subtle);",
                "Interface Appearance"
            }

            // Theme Toggle
            div {
                class: "flex items-center justify-between py-4",
                style: "border-bottom: 1px solid var(--border-subtle);", // Added separator for visual flow

                div {
                    div { class: "font-medium", style: "color: var(--text-primary);", "Dark Mode" }
                    div { class: "text-sm", style: "color: var(--text-secondary);", "Toggle application color theme" }
                }
                button {
                    onclick: move |_| {
                        let mut settings = app_state_theme.settings.write();
                        settings.theme = if dark_mode { "light".to_string() } else { "dark".to_string() };
                        if let Err(error) = save_settings(&settings) {
                            tracing::error!("Failed to save settings: {}", error);
                        }
                    },
                    class: "relative inline-flex h-7 w-12 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2",
                    style: format!(
                        "background-color: {}; --tw-ring-color: var(--accent-primary);",
                        if dark_mode { "var(--accent-primary)" } else { "var(--bg-active)" }
                    ),
                    span {
                        class: "inline-block h-5 w-5 transform rounded-full bg-white transition-transform shadow-sm",
                        style: format!(
                            "transform: translateX({});",
                            if dark_mode { "1.5rem" } else { "0.25rem" }
                        )
                    }
                }
            }

            // Font Size Selection
            div { class: "space-y-4", style: "gap: 1rem; display: flex; flex-direction: column;",
                div {
                    div { class: "font-medium", style: "color: var(--text-primary);", "Font Size" }
                    div { class: "text-sm", style: "color: var(--text-secondary);", "Adjust the text size of the chat interface" }
                }
                div { class: "grid grid-cols-3 gap-4", style: "gap: 1rem; display: grid; grid-template-columns: repeat(3, minmax(0, 1fr));",
                    for size in &["Small", "Medium", "Large"] {
                        button {
                            onclick: move |_| {
                                let mut settings = app_state_font_size.settings.write();
                                settings.font_size = size.to_lowercase();
                                if let Err(error) = save_settings(&settings) {
                                    tracing::error!("Failed to save settings: {}", error);
                                }
                            },
                            class: "p-4 rounded-xl border-2 text-center transition-all",
                            style: if selected_font_size == *size {
                                "border-color: var(--accent-primary); background-color: var(--bg-active); color: var(--accent-primary); box-shadow: var(--shadow-sm);"
                            } else {
                                "border-color: var(--border-subtle); background-color: var(--bg-input); color: var(--text-secondary);"
                            },
                            div { class: "font-semibold", "{size}" }
                            div {
                                class: "mt-1",
                                style: match *size {
                                    "Small" => "font-size: 0.875rem; color: var(--text-tertiary);",
                                    "Medium" => "font-size: 1rem; color: var(--text-tertiary);",
                                    "Large" => "font-size: 1.25rem; color: var(--text-tertiary);",
                                    _ => ""
                                },
                                "Aa"
                            }
                        }
                    }
                }
            }
        }
    }
}
