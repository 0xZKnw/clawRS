use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    // We could add timestamp, id, etc. later
}

#[component]
pub fn MessageBubble(message: Message) -> Element {
    let is_user = message.role == MessageRole::User;

    let container_class = if is_user {
        "flex flex-row-reverse items-start gap-4 mb-6 group"
    } else {
        "flex flex-row items-start gap-4 mb-6 group"
    };

    let bubble_class = if is_user {
        "bg-[var(--accent-primary)] text-[var(--accent-text)] rounded-2xl rounded-tr-sm px-5 py-3.5 shadow-md max-w-[85%] leading-relaxed"
    } else {
        "bg-[var(--bg-hover)] text-[var(--text-primary)] rounded-2xl rounded-tl-sm px-5 py-3.5 shadow-sm max-w-[85%] leading-relaxed border border-[var(--border-subtle)]"
    };

    rsx! {
        div { class: "{container_class}",
            // Avatar
            div {
                class: "flex-shrink-0 mt-1",
                div {
                    class: "w-8 h-8 rounded-full flex items-center justify-center shadow-sm text-xs font-bold " .to_string() +
                    if is_user { "bg-[var(--accent-hover)] text-white" } else { "bg-gradient-to-br from-[var(--accent-primary)] to-purple-600 text-white" },

                    if is_user {
                        "U"
                    } else {
                        svg { width: "16", height: "16", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", path { d: "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" } }
                    }
                }
            }

            // Message Content
            div {
                class: "flex flex-col " .to_string() + if is_user { "items-end" } else { "items-start" },

                div {
                    class: "{bubble_class}",
                    // We'll eventually use a Markdown component here
                    // For now, simple text with whitespace preservation
                    div {
                        class: "whitespace-pre-wrap break-words",
                        "{message.content}"
                    }
                }

                // Timestamp / Meta (Hidden by default, shown on hover)
                div {
                    class: "text-[10px] text-[var(--text-tertiary)] mt-1 opacity-0 group-hover:opacity-100 transition-opacity px-1",
                    if is_user { "You" } else { "LocaLM" }
                }
            }
        }
    }
}
