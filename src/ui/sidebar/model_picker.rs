use dioxus::prelude::*;
use crate::app::{AppState, ModelState};
use crate::storage::models::scan_models_directory;
use crate::ui::components::loading::Spinner;

#[component]
pub fn ModelPicker() -> Element {
    let app_state = use_context::<AppState>();
    let models_directory = app_state.settings.read().models_directory.clone();
    let models = use_signal(|| scan_models_directory(&models_directory).unwrap_or_default());

    let mut selected_model_path = use_signal(|| None);

    // Handlers
    let app_state_for_load = app_state.clone();
    let selected_model_path_for_load = selected_model_path.clone();
    let handle_load = move |_| {
        let mut app_state = app_state_for_load.clone();
        app_state.model_state.set(ModelState::Loading);
        let path = selected_model_path_for_load
            .read()
            .clone()
            .unwrap_or_default();
        let gpu_layers = app_state.settings.read().gpu_layers;
        spawn(async move {
            let result = {
                let mut engine = app_state.engine.lock().await;
                if !engine.is_initialized() {
                    if let Err(e) = engine.init() {
                        return app_state.model_state.set(ModelState::Error(e.to_string()));
                    }
                }
                engine.load_model(&path, gpu_layers)
            };
            match result {
                Ok(_info) => app_state.model_state.set(ModelState::Loaded(path)),
                Err(e) => app_state.model_state.set(ModelState::Error(e.to_string())),
            }
        });
    };

    let app_state_for_unload = app_state.clone();
    let handle_unload = move |_| {
        let mut app_state = app_state_for_unload.clone();
        spawn(async move {
            let mut engine = app_state.engine.lock().await;
            engine.unload_model();
        });
        app_state.model_state.set(ModelState::NotLoaded);
    };

    let app_state_for_refresh = app_state.clone();
    let mut models_for_refresh = models.clone();
    let handle_refresh = move |_| {
        let models_directory = app_state_for_refresh
            .settings
            .read()
            .models_directory
            .clone();
        models_for_refresh.set(scan_models_directory(&models_directory).unwrap_or_default());
    };

    rsx! {
        div {
            class: "flex flex-col p-4 border-b border-[var(--border-subtle)] gap-3 bg-[var(--bg-sidebar)]",
            
            // Header with Refresh
            div {
                class: "flex items-center justify-between",
                span {
                    class: "text-[10px] uppercase tracking-wider text-[var(--text-tertiary)] font-bold select-none",
                    "Model Selection"
                }
                button {
                    onclick: handle_refresh,
                    class: "text-[var(--text-tertiary)] hover:text-[var(--text-primary)] transition-colors p-1 rounded-sm hover:bg-[var(--bg-hover)]",
                    title: "Rescan models",
                    svg {
                        class: "w-3 h-3",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "M23 4v6h-6" }
                        path { d: "M1 20v-6h6" }
                        path { d: "M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" }
                    }
                }
            }

            // Main Content Area
            if models.read().is_empty() {
                div {
                    class: "flex flex-col items-center justify-center p-4 border border-dashed border-[var(--border-subtle)] rounded-md gap-2",
                    span { class: "text-sm text-[var(--text-secondary)]", "No models found" }
                    span { class: "text-[10px] text-[var(--text-tertiary)] text-center", "Place .gguf files in /models" }
                }
            } else {
                div {
                    class: "flex flex-col gap-2",
                    
                    // Model Selector
                    div {
                        class: "relative group",
                        select {
                            class: "w-full appearance-none bg-[var(--bg-input)] border border-[var(--border-subtle)] text-[var(--text-primary)] text-sm rounded-md py-2 pl-3 pr-8 focus:outline-none focus:border-[var(--accent-primary)] focus:ring-1 focus:ring-[var(--accent-primary)] transition-all font-mono",
                            disabled: matches!(*app_state.model_state.read(), ModelState::Loading | ModelState::Loaded(_)),
                            onchange: move |evt| selected_model_path.set(Some(evt.value())),
                            value: selected_model_path.read().clone().unwrap_or_default(),
                            
                            for model in models.read().iter() {
                                option {
                                    value: "{model.path.to_string_lossy()}",
                                    "{model.filename}"
                                }
                            }
                        }
                        // Custom Chevron
                        div {
                            class: "absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none text-[var(--text-tertiary)] group-hover:text-[var(--text-secondary)]",
                            svg {
                                class: "w-4 h-4",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                polyline { points: "6 9 12 15 18 9" }
                            }
                        }
                    }

                    // Metadata display (Size)
                    if let Some(path) = selected_model_path.read().as_ref() {
                        if let Some(model) = models.read().iter().find(|m| m.path.to_string_lossy() == *path) {
                            div {
                                class: "flex justify-end",
                                span {
                                    class: "text-[10px] text-[var(--text-tertiary)] font-mono",
                                    "{model.size_string()}"
                                }
                            }
                        }
                    }

                    // Actions & Status
                    match *app_state.model_state.read() {
                        ModelState::NotLoaded => rsx! {
                            button {
                                onclick: handle_load,
                                class: "w-full flex items-center justify-center gap-2 bg-[var(--accent-primary)] hover:bg-[var(--accent-hover)] text-white text-sm font-medium py-2 rounded-md transition-all active:scale-[0.98]",
                                svg {
                                    class: "w-4 h-4",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    path { d: "M5 12h14" }
                                    path { d: "M12 5l7 7-7 7" }
                                }
                                "Load Model"
                            }
                        },
                        ModelState::Loading => rsx! {
                            div {
                                class: "w-full flex items-center justify-center gap-3 bg-[var(--bg-subtle)] border border-[var(--border-subtle)] py-2 rounded-md",
                                Spinner { size: 16 }
                                span { class: "text-sm text-[var(--text-secondary)]", "Loading..." }
                            }
                        },
                        ModelState::Loaded(_) => rsx! {
                            div {
                                class: "flex items-center gap-2",
                                div {
                                    class: "flex-1 flex items-center gap-2 px-3 py-2 bg-[var(--bg-success-subtle)] border border-[var(--border-success-subtle)] rounded-md",
                                    div { class: "w-2 h-2 rounded-full bg-[var(--success)] animate-pulse" }
                                    span { class: "text-xs font-medium text-[var(--text-success)]", "Active" }
                                }
                                button {
                                    onclick: handle_unload,
                                    class: "px-3 py-2 text-sm text-[var(--text-secondary)] border border-[var(--border-subtle)] rounded-md hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] transition-colors",
                                    title: "Unload Model",
                                    svg {
                                        class: "w-4 h-4",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        path { d: "M18.36 6.64a9 9 0 1 1-12.73 0" }
                                        line { x1: "12", y1: "2", x2: "12", y2: "12" }
                                    }
                                }
                            }
                        },
                        ModelState::Error(ref msg) => rsx! {
                            div {
                                class: "w-full p-2 bg-[var(--bg-error-subtle)] border border-[var(--border-error-subtle)] rounded-md text-xs text-[var(--text-error)]",
                                "{msg}"
                            }
                        }
                    }
                }
            }
        }
    }
}
