pub mod conversation_list;
pub mod model_picker;

use crate::ui::sidebar::conversation_list::ConversationList;
use crate::ui::sidebar::model_picker::ModelPicker;
use dioxus::prelude::*;

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside {
            class: "flex flex-col w-[260px] h-full bg-[var(--bg-sidebar)] border-r border-[var(--border-subtle)] transition-colors duration-300",

            // Model Selector
            ModelPicker {}

            // Header: New Chat
            div {
                class: "p-3",
                button {
                    class: "w-full flex items-center gap-3 px-3 py-3 text-sm text-[var(--text-primary)] border border-[var(--border-subtle)] rounded-md hover:bg-[var(--bg-hover)] transition-colors duration-200 text-left shadow-sm",

                    svg {
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "M12 5v14M5 12h14" }
                    }
                    "New Chat"
                }
            }

            // Conversation List
            ConversationList {}

            // Footer: User / Settings
            div {
                class: "p-3 border-t border-[var(--border-subtle)]",

                // Settings Button
                button {
                    class: "w-full flex items-center gap-3 px-3 py-3 text-sm text-[var(--text-secondary)] rounded-md hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] transition-colors duration-200",

                    svg {
                        width: "16",
                        height: "16",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        circle { cx: "12", cy: "12", r: "3" }
                        path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 5 9.4a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" }
                    }
                    "Settings"
                }
            }
        }
    }
}
