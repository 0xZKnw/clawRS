use dioxus::prelude::*;

#[component]
pub fn ChatInput(
    on_send: EventHandler<String>,
    on_stop: EventHandler<()>,
    is_generating: bool,
) -> Element {
    let mut text = use_signal(|| String::new());

    let handle_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Escape && is_generating {
            on_stop.call(());
        } else if evt.key() == Key::Enter && !evt.modifiers().contains(Modifiers::SHIFT) {
            // Enter without Shift = Send
            evt.prevent_default();
            if !is_generating && !text().trim().is_empty() {
                on_send.call(text());
                text.set(String::new());
            }
        }
        // Shift+Enter = new line (default behavior, no need to handle)
    };

    let handle_send_click = move |_| {
        if !is_generating && !text().trim().is_empty() {
            on_send.call(text());
            text.set(String::new());
        }
    };

    rsx! {
        div {
            class: "w-full p-4 bg-[var(--bg-main)]",

            div {
                class: "relative flex items-end gap-3 max-w-4xl mx-auto p-2 bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-2xl shadow-lg hover:border-[var(--border-hover)] focus-within:border-[var(--border-focus)] focus-within:ring-1 focus-within:ring-[var(--border-focus)] transition-all duration-200",

                // Textarea container
                div {
                    class: "relative flex-1",

                    textarea {
                        class: "w-full max-h-48 min-h-[52px] py-3 px-3 bg-transparent border-none outline-none text-[var(--text-primary)] resize-none placeholder-[var(--text-tertiary)] text-base font-sans leading-relaxed",
                        placeholder: "Ask anything...",
                        value: "{text}",
                        oninput: move |evt| text.set(evt.value()),
                        onkeydown: handle_keydown,
                        disabled: is_generating,
                    }
                }

                // Send / Stop Button
                div {
                    class: "flex-shrink-0 pb-1.5 pr-1.5",
                    if is_generating {
                        button {
                            onclick: move |_| on_stop.call(()),
                            class: "p-2.5 rounded-xl bg-[var(--bg-subtle)] text-[var(--text-secondary)] hover:bg-[var(--bg-error-subtle)] hover:text-[var(--text-error)] transition-colors border border-[var(--border-subtle)]",
                            title: "Stop generating (Esc)",
                            div {
                                class: "w-3 h-3 bg-current rounded-sm"
                            }
                        }
                    } else {
                        button {
                            onclick: handle_send_click,
                            disabled: text().trim().is_empty(),
                            class: "p-2.5 rounded-xl bg-[var(--accent-primary)] text-[var(--accent-text)] hover:bg-[var(--accent-hover)] disabled:opacity-30 disabled:cursor-not-allowed transition-all shadow-md hover:shadow-glow active:scale-95 disabled:shadow-none",
                            title: "Send message (Enter)",
                            svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.5", stroke_linecap: "round", stroke_linejoin: "round", line { x1: "22", y1: "2", x2: "11", y2: "13" }, polygon { points: "22 2 15 22 11 13 2 9 22 2" } }
                        }
                    }
                }
            }

            // Footer text
            div {
                class: "text-center mt-3 text-[10px] text-[var(--text-tertiary)] select-none",
                "LocaLM runs entirely on your device. "
                span { class: "hidden sm:inline", "Conversations are private and secure." }
            }
        }
    }
}
