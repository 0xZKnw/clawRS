use dioxus::prelude::*;
use crate::app::{AppState, ModelState};
use crate::storage::models::scan_models_directory;
use crate::ui::components::loading::Spinner;

#[component]
pub fn ModelPicker() -> Element {
    let app_state = use_context::<AppState>();
    let models_directory = app_state.settings.read().models_directory.clone();
    
    let mut models = use_signal(Vec::new);
    let mut selected_model_path = use_signal(|| None::<String>);
    
    let models_directory_clone = models_directory.clone();
    use_effect(move || {
        let found_models = scan_models_directory(&models_directory_clone).unwrap_or_default();
        // Pre-select first model if available and nothing selected yet
        if selected_model_path.read().is_none() {
            if let Some(first_model) = found_models.first() {
                let path_str = first_model.path.to_string_lossy().to_string();
                tracing::debug!("Pre-selecting first model: {}", path_str);
                selected_model_path.set(Some(path_str));
            }
        }
        models.set(found_models);
    });

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
            class: "flex flex-col p-4 border-b border-[var(--border-subtle)] gap-4 bg-[var(--bg-sidebar)]",
            
            // Header with Refresh
            div {
                class: "flex items-center justify-between",
                span {
                    class: "text-[10px] uppercase tracking-wider text-[var(--text-tertiary)] font-bold select-none",
                    "Active Model"
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
                    class: "flex flex-col items-center justify-center p-4 border border-dashed border-[var(--border-subtle)] rounded-lg gap-2 bg-[var(--bg-subtle)]",
                    span { class: "text-sm text-[var(--text-secondary)] font-medium", "No models found" }
                    span { class: "text-[10px] text-[var(--text-tertiary)] text-center", "Place .gguf files in /models" }
                }
            } else {
                div {
                    class: "flex flex-col gap-3",
                    
                    // Model Selector
                    div {
                        class: "relative group",
                        select {
                            class: "w-full appearance-none bg-[var(--bg-input)] border border-[var(--border-subtle)] text-[var(--text-primary)] text-sm rounded-lg py-2.5 pl-3 pr-8 focus:outline-none focus:border-[var(--accent-primary)] focus:ring-1 focus:ring-[var(--accent-primary)] transition-all font-medium truncate disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer hover:border-[var(--border-hover)]",
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
                            class: "absolute right-2.5 top-1/2 -translate-y-1/2 pointer-events-none text-[var(--text-tertiary)] group-hover:text-[var(--text-secondary)]",
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

                    // Metadata display (Size) - cleaner look
                    if let Some(path) = selected_model_path.read().as_ref() {
                        if let Some(model) = models.read().iter().find(|m| m.path.to_string_lossy() == *path) {
                            div {
                                class: "flex justify-end",
                                span {
                                    class: "px-1.5 py-0.5 rounded text-[10px] font-mono bg-[var(--bg-subtle)] text-[var(--text-tertiary)]",
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
                                class: "w-full flex items-center justify-center gap-2 bg-[var(--bg-surface)] border border-[var(--border-subtle)] hover:border-[var(--accent-primary)] hover:text-[var(--accent-primary)] text-[var(--text-secondary)] text-sm font-medium py-2.5 rounded-lg transition-all active:scale-[0.98] shadow-sm",
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
                                class: "w-full flex items-center justify-center gap-3 bg-[var(--bg-subtle)] border border-[var(--border-subtle)] py-2.5 rounded-lg",
                                Spinner { size: 16 }
                                span { class: "text-xs font-medium text-[var(--text-secondary)]", "Loading into memory..." }
                            }
                        },
                        ModelState::Loaded(_) => rsx! {
                            div {
                                class: "flex items-center gap-2",
                                div {
                                    class: "flex-1 flex items-center gap-2 px-3 py-2.5 bg-[var(--bg-success-subtle)] border border-[var(--border-success-subtle)] rounded-lg",
                                    div { class: "w-1.5 h-1.5 rounded-full bg-[var(--success)] animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.4)]" }
                                    span { class: "text-xs font-medium text-[var(--text-success)]", "Ready" }
                                }
                                button {
                                    onclick: handle_unload,
                                    class: "px-3 py-2.5 text-sm text-[var(--text-secondary)] border border-[var(--border-subtle)] rounded-lg hover:bg-[var(--bg-error-subtle)] hover:border-[var(--border-error-subtle)] hover:text-[var(--text-error)] transition-colors",
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
                                class: "w-full p-2 bg-[var(--bg-error-subtle)] border border-[var(--border-error-subtle)] rounded-lg text-xs text-[var(--text-error)]",
                                "{msg}"
                            }
                        }
                    }
                }
            }
        }
    }
}
