use crate::{gpui, ContextMenuComponentNode, ElementRenderContext};

/// A secondary click on the content opens GPUI Component's popup menu at
/// the pointer; choosing an item reports its value through `phx-select`.
/// Primary clicks still reach the content.
#[cfg(feature = "components")]
pub(crate) fn render(
    node: ContextMenuComponentNode,
    context: &mut ElementRenderContext<'_, '_>,
) -> gpui::AnyElement {
    use crate::element::apply_generated_render_styles;
    use crate::{push_event, ElementNode, EventValue, InputKind, NativeEvent};
    use gpui::{InteractiveElement, IntoElement, ParentElement};
    use gpui_component::menu::{ContextMenuExt, PopupMenuItem};

    let mut items = Vec::new();
    let mut content = Vec::new();
    for child in node.children {
        match child {
            ElementNode::DropdownMenuItemComponent(item) => {
                items.push((item.value, item.label, item.disabled, item.checked))
            }
            other => content.push(other.render(context)),
        }
    }

    let runtime = context.runtime.clone();
    let window_id = context.window_id;
    let select = node.select;

    apply_generated_render_styles(gpui::div(), node.style)
        .id(node.id)
        .children(content)
        .context_menu(move |mut menu, _window, _cx| {
            for (value, label, disabled, checked) in &items {
                let runtime = runtime.clone();
                let select = select.clone();
                let value = value.clone();
                menu = menu.item(
                    PopupMenuItem::new(label.clone())
                        .disabled(*disabled)
                        .checked(*checked)
                        .on_click(move |_event, _window, _cx| {
                            if let Some(event) = select.clone() {
                                let _ = push_event(
                                    &runtime,
                                    NativeEvent::Input {
                                        kind: InputKind::Select,
                                        window_id,
                                        event,
                                        value: Some(EventValue::String(value.clone())),
                                    },
                                );
                            }
                        }),
                );
            }
            menu
        })
        .into_any_element()
}

#[cfg(not(feature = "components"))]
pub(crate) fn render(
    node: ContextMenuComponentNode,
    context: &mut ElementRenderContext<'_, '_>,
) -> gpui::AnyElement {
    super::render_component_fallback(node.style, None, node.children, context)
}
