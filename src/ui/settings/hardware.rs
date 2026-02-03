use crate::app::AppState;
use crate::storage::settings::save_settings;
use dioxus::prelude::*;

pub fn HardwareSettings() -> Element {
    let app_state = use_context::<AppState>();
    let settings = app_state.settings.read().clone();
    let gpu_layers = settings.gpu_layers;
    let models_dir = settings.models_directory.to_string_lossy().to_string();
    let mut app_state_gpu_layers = app_state.clone();

    // Mock Hardware Info
    let gpu_name = "NVIDIA GeForce RTX 4090 (Mock)";
    let vram_total_gb = 24.0;
    let vram_used_gb = 4.2;
    let vram_percent = (vram_used_gb / vram_total_gb) * 100.0;

    rsx! {
        div {
            class: "space-y-8 max-w-3xl mx-auto animate-fade-in",
            style: "padding-bottom: 2rem;",

            h3 {
                class: "text-xl font-semibold pb-2",
                style: "color: var(--text-primary); border-bottom: 1px solid var(--border-subtle);",
                "Hardware Acceleration"
            }

             // GPU Info Card
             div {
                 class: "p-6 rounded-xl border shadow-sm",
                 style: "background: linear-gradient(135deg, var(--bg-hover) 0%, var(--bg-main) 100%); border-color: var(--border-subtle);",

                div { class: "flex items-start space-x-4", style: "gap: 1rem;",
                    // Chip Icon
                    div {
                        class: "p-3 rounded-lg",
                        style: "background-color: var(--bg-active); color: var(--accent-primary);",
                        svg { class: "w-6 h-6", style: "width: 1.5rem; height: 1.5rem;", fill: "none", stroke: "currentColor", view_box: "0 0 24 24", stroke_width: "2",
                            path { d: "M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" }
                        }
                    }
                    div { class: "flex-1",
                        div { class: "font-bold text-lg", style: "color: var(--text-primary);", "{gpu_name}" }
                        div { class: "mt-4 space-y-1",
                            div { class: "flex justify-between text-sm", style: "color: var(--text-secondary); margin-bottom: 0.5rem;",
                                span { "VRAM Usage" }
                                span { "{vram_used_gb} GB / {vram_total_gb} GB" }
                            }
                            // Progress Bar
                            div {
                                class: "w-full rounded-full h-2.5 overflow-hidden",
                                style: "background-color: var(--bg-active);",
                                div {
                                    class: "h-2.5 rounded-full transition-all duration-500",
                                    style: "width: {vram_percent}%; background-color: var(--accent-primary);"
                                }
                            }
                        }
                    }
                }
             }

             // GPU Layers Control
             div { class: "space-y-3", style: "gap: 0.75rem; display: flex; flex-direction: column;",
                div { class: "flex justify-between items-center",
                    label { class: "font-medium", style: "color: var(--text-primary);", "GPU Layers" }
                    span {
                        class: "text-sm font-mono px-2 py-1 rounded",
                        style: "background-color: var(--bg-hover); color: var(--text-secondary);",
                        "{gpu_layers}"
                    }
                }
                input {
                    r#type: "range",
                    min: "0",
                    max: "99",
                    value: "{gpu_layers}",
                    oninput: move |e| {
                        let value = e.value().parse().unwrap_or(0);
                        let mut settings = app_state_gpu_layers.settings.write();
                        settings.gpu_layers = value;
                        if let Err(error) = save_settings(&settings) {
                            tracing::error!("Failed to save settings: {}", error);
                        }
                    },
                    class: "w-full h-2 rounded-lg appearance-none cursor-pointer",
                    style: "background-color: var(--bg-active); accent-color: var(--accent-primary);"
                }
                p { class: "text-xs", style: "color: var(--text-tertiary);",
                    "Number of model layers to offload to the GPU. Higher values improve inference speed but require more VRAM."
                }
             }

             // Models Directory Input
             div { class: "space-y-2", style: "gap: 0.5rem; display: flex; flex-direction: column;",
                label { class: "font-medium block", style: "color: var(--text-primary);", "Models Directory" }
                div { class: "flex space-x-2", style: "gap: 0.5rem;",
                    input {
                        r#type: "text",
                        readonly: true,
                        value: "{models_dir}",
                        class: "flex-1 p-3 rounded-lg border cursor-not-allowed",
                        style: "background-color: var(--bg-hover); color: var(--text-secondary); border-color: var(--border-subtle);"
                    }
                    button {
                        class: "px-4 py-2 border rounded-lg font-medium transition-colors shadow-sm",
                        style: "background-color: var(--bg-input); color: var(--text-primary); border-color: var(--border-subtle);",
                        "Browse..."
                    }
                }
                p { class: "text-xs", style: "color: var(--text-tertiary);",
                    "Location where model files (.gguf) are stored."
                }
             }
        }
    }
}
