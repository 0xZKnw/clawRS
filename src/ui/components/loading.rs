use dioxus::prelude::*;

#[component]
pub fn Spinner(props: SpinnerProps) -> Element {
    let size = props.size.unwrap_or(24);
    let color = props.color.unwrap_or("var(--accent-primary)".to_string());

    rsx! {
        div {
            class: "spinner",
            style: "width: {size}px; height: {size}px; border: 2px solid var(--bg-active); border-top-color: {color}; border-radius: 50%; animation: spin 1s linear infinite;",
        }
        style {
            "@keyframes spin {{ to {{ transform: rotate(360deg); }} }}"
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SpinnerProps {
    #[props(optional)]
    pub size: Option<i32>,
    #[props(optional)]
    pub color: Option<String>,
}

#[component]
pub fn Skeleton(props: SkeletonProps) -> Element {
    let width = props.width.unwrap_or("100%".to_string());
    let height = props.height.unwrap_or("1rem".to_string());
    let class = props.class.unwrap_or_default();

    rsx! {
        div {
            class: "skeleton {class}",
            style: "width: {width}; height: {height}; background-color: var(--bg-active); border-radius: var(--radius-sm); animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;",
        }
        style {
            "@keyframes pulse {{ 0%, 100% {{ opacity: 1; }} 50% {{ opacity: .5; }} }}"
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SkeletonProps {
    #[props(optional)]
    pub width: Option<String>,
    #[props(optional)]
    pub height: Option<String>,
    #[props(optional)]
    pub class: Option<String>,
}
