#![allow(non_snake_case)]

pub mod appearance;
pub mod hardware;
pub mod inference;

use crate::ui::settings::appearance::AppearanceSettings;
use crate::ui::settings::hardware::HardwareSettings;
use crate::ui::settings::inference::InferenceSettings;
use dioxus::prelude::*;

#[derive(PartialEq, Clone, Copy)]
enum SettingsTab {
    Inference,
    Hardware,
    Appearance,
}

pub fn Settings() -> Element {
    let mut active_tab = use_signal(|| SettingsTab::Inference);

    rsx! {
        div {
            class: "flex flex-col h-full font-sans",
            style: "background-color: var(--bg-main); color: var(--text-primary);",

            // Header
            div {
                class: "flex-none px-8 py-6",
                style: "background-color: var(--bg-main); border-bottom: 1px solid var(--border-subtle);",

                h2 {
                    class: "text-2xl font-bold tracking-tight mb-6",
                    style: "color: var(--text-primary);",
                    "Settings"
                }

                // Tabs
                div {
                    class: "flex",
                    style: "gap: 0.25rem; border-bottom: 1px solid var(--border-subtle);",

                    TabButton {
                        active: active_tab() == SettingsTab::Inference,
                        onclick: move |_| active_tab.set(SettingsTab::Inference),
                        label: "Inference",
                        icon: rsx! {
                            svg { class: "w-5 h-5 mr-2", style: "margin-right: 0.5rem; width: 1.25rem; height: 1.25rem;", fill: "none", stroke: "currentColor", view_box: "0 0 24 24", stroke_width: "2",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M13 10V3L4 14h7v7l9-11h-7z" }
                            }
                        }
                    }
                    TabButton {
                        active: active_tab() == SettingsTab::Hardware,
                        onclick: move |_| active_tab.set(SettingsTab::Hardware),
                        label: "Hardware",
                        icon: rsx! {
                            svg { class: "w-5 h-5 mr-2", style: "margin-right: 0.5rem; width: 1.25rem; height: 1.25rem;", fill: "none", stroke: "currentColor", view_box: "0 0 24 24", stroke_width: "2",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" }
                            }
                        }
                    }
                    TabButton {
                        active: active_tab() == SettingsTab::Appearance,
                        onclick: move |_| active_tab.set(SettingsTab::Appearance),
                        label: "Appearance",
                        icon: rsx! {
                            svg { class: "w-5 h-5 mr-2", style: "margin-right: 0.5rem; width: 1.25rem; height: 1.25rem;", fill: "none", stroke: "currentColor", view_box: "0 0 24 24", stroke_width: "2",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" }
                            }
                        }
                    }
                }
            }

            // Content Area
            div {
                class: "flex-1 overflow-y-auto px-8 py-6",
                match active_tab() {
                    SettingsTab::Inference => rsx! { InferenceSettings {} },
                    SettingsTab::Hardware => rsx! { HardwareSettings {} },
                    SettingsTab::Appearance => rsx! { AppearanceSettings {} },
                }
            }
        }
    }
}

#[component]
fn TabButton(
    active: bool,
    onclick: EventHandler<MouseEvent>,
    label: String,
    icon: Element,
) -> Element {
    let (bg, color, border) = if active {
        (
            "var(--bg-active)",
            "var(--accent-primary)",
            "2px solid var(--accent-primary)",
        )
    } else {
        (
            "transparent",
            "var(--text-secondary)",
            "2px solid transparent",
        )
    };

    rsx! {
        button {
            class: "flex items-center px-6 py-3 transition-all duration-200 focus:outline-none",
            style: "background-color: {bg}; color: {color}; border-bottom: {border}; cursor: pointer; border-radius: 4px 4px 0 0;",
            onclick: onclick,
            {icon}
            "{label}"
        }
    }
}
