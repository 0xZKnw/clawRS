use crate::app::AppState;
use crate::storage::settings::save_settings;
use dioxus::prelude::*;

pub fn InferenceSettings() -> Element {
    let app_state = use_context::<AppState>();
    let settings = app_state.settings.read().clone();
    let temperature = settings.temperature;
    let top_p = settings.top_p;
    let top_k = settings.top_k;
    let max_tokens = settings.max_tokens;
    let context_size = settings.context_size;
    let system_prompt = settings.system_prompt.clone();
    let mut app_state_temperature = app_state.clone();
    let mut app_state_top_p = app_state.clone();
    let mut app_state_top_k = app_state.clone();
    let mut app_state_max_tokens = app_state.clone();
    let mut app_state_context_size = app_state.clone();
    let mut app_state_system_prompt = app_state.clone();

    rsx! {
        div {
            class: "space-y-8 max-w-3xl mx-auto animate-fade-in",
            style: "padding-bottom: 2rem;",

            // Section: Generation Parameters
            div { class: "space-y-6",
                h3 {
                    class: "text-xl font-semibold pb-2",
                    style: "color: var(--text-primary); border-bottom: 1px solid var(--border-subtle);",
                    "Generation Parameters"
                }

                // Temperature Slider
                div {
                    class: "space-y-3",
                    style: "gap: 0.75rem; display: flex; flex-direction: column;",

                    div { class: "flex justify-between items-center",
                        label { class: "font-medium", style: "color: var(--text-primary);", "Temperature" }
                        span {
                            class: "text-sm font-mono px-2 py-1 rounded",
                            style: "background-color: var(--bg-hover); color: var(--text-secondary);",
                            "{temperature:.2}"
                        }
                    }
                    input {
                        r#type: "range",
                        min: "0",
                        max: "2",
                        step: "0.1",
                        value: "{temperature}",
                        oninput: move |e| {
                            let value = e.value().parse().unwrap_or(0.7);
                            let mut settings = app_state_temperature.settings.write();
                            settings.temperature = value;
                            if let Err(error) = save_settings(&settings) {
                                tracing::error!("Failed to save settings: {}", error);
                            }
                        },
                        class: "w-full h-2 rounded-lg appearance-none cursor-pointer",
                        style: "background-color: var(--bg-active); accent-color: var(--accent-primary);"
                    }
                    p { class: "text-xs", style: "color: var(--text-tertiary);",
                        "Controls randomness. Higher values (e.g., 1.0) make output more random, while lower values (e.g., 0.2) make it more focused and deterministic."
                    }
                }

                // Top P Slider
                div {
                    class: "space-y-3",
                    style: "gap: 0.75rem; display: flex; flex-direction: column;",

                    div { class: "flex justify-between items-center",
                        label { class: "font-medium", style: "color: var(--text-primary);", "Top P" }
                        span {
                            class: "text-sm font-mono px-2 py-1 rounded",
                            style: "background-color: var(--bg-hover); color: var(--text-secondary);",
                            "{top_p:.2}"
                        }
                    }
                    input {
                        r#type: "range",
                        min: "0",
                        max: "1",
                        step: "0.05",
                        value: "{top_p}",
                        oninput: move |e| {
                            let value = e.value().parse().unwrap_or(0.9);
                            let mut settings = app_state_top_p.settings.write();
                            settings.top_p = value;
                            if let Err(error) = save_settings(&settings) {
                                tracing::error!("Failed to save settings: {}", error);
                            }
                        },
                        class: "w-full h-2 rounded-lg appearance-none cursor-pointer",
                        style: "background-color: var(--bg-active); accent-color: var(--accent-primary);"
                    }
                    p { class: "text-xs", style: "color: var(--text-tertiary);",
                        "Nucleus sampling. Considers the smallest set of tokens whose cumulative probability exceeds the threshold P."
                    }
                }

                // Top K Input
                div { class: "space-y-2", style: "gap: 0.5rem; display: flex; flex-direction: column;",
                    label { class: "font-medium block", style: "color: var(--text-primary);", "Top K" }
                    input {
                        r#type: "number",
                        min: "0",
                        max: "100",
                        value: "{top_k}",
                        oninput: move |e| {
                            let value = e.value().parse().unwrap_or(40);
                            let mut settings = app_state_top_k.settings.write();
                            settings.top_k = value;
                            if let Err(error) = save_settings(&settings) {
                                tracing::error!("Failed to save settings: {}", error);
                            }
                        },
                        class: "w-full p-3 rounded-lg border focus:ring-2 transition-all",
                        style: "background-color: var(--bg-input); color: var(--text-primary); border-color: var(--border-subtle); outline-color: var(--border-focus);"
                    }
                    p { class: "text-xs", style: "color: var(--text-tertiary);",
                        "Limits the next token selection to the K most likely tokens."
                    }
                }
            }

            // Section: Model Configuration
            div { class: "space-y-6",
                h3 {
                    class: "text-xl font-semibold pb-2",
                    style: "color: var(--text-primary); border-bottom: 1px solid var(--border-subtle);",
                    "Model Configuration"
                }

                // Max Tokens Input
                div { class: "space-y-2", style: "gap: 0.5rem; display: flex; flex-direction: column;",
                    label { class: "font-medium block", style: "color: var(--text-primary);", "Max Tokens" }
                    input {
                        r#type: "number",
                        min: "1",
                        max: "4096",
                        value: "{max_tokens}",
                        oninput: move |e| {
                            let value = e.value().parse().unwrap_or(2048);
                            let mut settings = app_state_max_tokens.settings.write();
                            settings.max_tokens = value;
                            if let Err(error) = save_settings(&settings) {
                                tracing::error!("Failed to save settings: {}", error);
                            }
                        },
                        class: "w-full p-3 rounded-lg border focus:ring-2 transition-all",
                        style: "background-color: var(--bg-input); color: var(--text-primary); border-color: var(--border-subtle); outline-color: var(--border-focus);"
                    }
                }

                // Context Size Dropdown
                div { class: "space-y-2", style: "gap: 0.5rem; display: flex; flex-direction: column;",
                    label { class: "font-medium block", style: "color: var(--text-primary);", "Context Window" }
                    select {
                        value: "{context_size}",
                        onchange: move |e| {
                            let value = e.value().parse().unwrap_or(4096);
                            let mut settings = app_state_context_size.settings.write();
                            settings.context_size = value;
                            if let Err(error) = save_settings(&settings) {
                                tracing::error!("Failed to save settings: {}", error);
                            }
                        },
                        class: "w-full p-3 rounded-lg border focus:ring-2 transition-all",
                        style: "background-color: var(--bg-input); color: var(--text-primary); border-color: var(--border-subtle); outline-color: var(--border-focus);",
                        option { value: "2048", "2048 Tokens" }
                        option { value: "4096", "4096 Tokens (Default)" }
                        option { value: "8192", "8192 Tokens" }
                        option { value: "16384", "16384 Tokens" }
                    }
                }

                // System Prompt Textarea
                div { class: "space-y-2", style: "gap: 0.5rem; display: flex; flex-direction: column;",
                    label { class: "font-medium block", style: "color: var(--text-primary);", "System Prompt" }
                    textarea {
                        value: "{system_prompt}",
                        oninput: move |e| {
                            let value = e.value();
                            let mut settings = app_state_system_prompt.settings.write();
                            settings.system_prompt = value;
                            if let Err(error) = save_settings(&settings) {
                                tracing::error!("Failed to save settings: {}", error);
                            }
                        },
                        class: "w-full p-3 rounded-lg border focus:ring-2 transition-all h-32 resize-y font-sans",
                        style: "background-color: var(--bg-input); color: var(--text-primary); border-color: var(--border-subtle); outline-color: var(--border-focus);",
                        placeholder: "Enter system prompt..."
                    }
                    p { class: "text-xs", style: "color: var(--text-tertiary);",
                        "The initial instructions given to the model to define its behavior and persona."
                    }
                }
            }
        }
    }
}
