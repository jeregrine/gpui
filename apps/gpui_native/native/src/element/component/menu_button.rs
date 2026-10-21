use crate::{gpui, ElementRenderContext, MenuButtonComponentNode};

/// A button whose press opens the context menu panel below it. The press stops
/// there, so the button can sit inside a clickable row; pressing it again
/// closes the menu. While the menu is open the button stays visible, even when
/// its style only shows it while a group is hovered.
#[cfg(feature = "components")]
pub(crate) fn render(
    node: MenuButtonComponentNode,
    context: &mut ElementRenderContext<'_, '_>,
) -> gpui::AnyElement {
    use super::context_menu::{open, split_items, Menu};
    use crate::element::apply_generated_render_styles;
    use gpui::prelude::FluentBuilder;
    use gpui::{
        anchored, deferred, div, px, Anchor, InteractiveElement, IntoElement, MouseButton,
        ParentElement, Role, StatefulInteractiveElement, Styled,
    };

    let (items, content) = split_items(node.children, context);
    let menu = Menu::new(&node.id, items, node.select, context);
    let state = menu.state.clone();
    let is_open = menu.position.is_some();
    let top_right = node.anchor.as_deref() == Some("top_right");

    let mut element = apply_generated_render_styles(div(), node.style)
        .id(node.id.clone())
        .role(Role::Button)
        .aria_label(node.label)
        .aria_expanded(is_open)
        .children(content)
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            cx.stop_propagation();
            // The press that just closed the menu from outside landed here.
            let toggled_closed = state.update(cx, |menu, _cx| {
                menu.closed_at.take() == Some(event.position)
            });
            if !toggled_closed {
                open(&state, event.position, window, cx);
            }
        });

    if is_open {
        element = element.visible().child(
            div()
                .absolute()
                .top_full()
                .mt(px(2.))
                .map(|corner| {
                    if top_right {
                        corner.right_0()
                    } else {
                        corner.left_0()
                    }
                })
                .child(
                    deferred(
                        anchored()
                            .anchor(if top_right {
                                Anchor::TopRight
                            } else {
                                Anchor::TopLeft
                            })
                            .snap_to_window_with_margin(px(8.))
                            .child(menu.panel(context)),
                    )
                    .with_priority(1),
                ),
        );
    }

    element.into_any_element()
}

#[cfg(not(feature = "components"))]
pub(crate) fn render(
    node: MenuButtonComponentNode,
    context: &mut ElementRenderContext<'_, '_>,
) -> gpui::AnyElement {
    super::render_component_fallback(node.style, None, node.children, context)
}

#[cfg(all(test, feature = "components"))]
mod tests {
    use crate::gpui;
    use gpui::{point, px, size, IntoElement, Modifiers, VisualTestContext};

    fn button() -> crate::ElementNode {
        let item = |value: &str| {
            crate::ElementNode::DropdownMenuItemComponent(crate::DropdownMenuItemComponentNode {
                style: crate::StyleAttrs::default(),
                value: value.to_string(),
                label: value.to_string(),
                disabled: false,
                checked: false,
            })
        };
        let mut style = crate::StyleAttrs::default();
        style.width = Some(px(20.0).into());
        style.height = Some(px(20.0).into());
        crate::ElementNode::MenuButtonComponent(crate::MenuButtonComponentNode {
            style,
            id: "more".to_string(),
            label: "More".to_string(),
            anchor: None,
            children: vec![item("rename"), item("remove")],
            select: Some("picked".to_string()),
        })
    }

    fn render(cx: &mut gpui::TestAppContext) -> (crate::SharedRuntime, &mut VisualTestContext) {
        cx.update(gpui_component::init);
        let runtime = std::sync::Arc::new(crate::runtime::RuntimeState::new());
        let state = std::sync::Arc::new(crate::WindowState::new(button(), Vec::new()));
        let runtime_for_view = runtime.clone();
        let (view, cx) = cx.add_window_view(move |_window, _cx| {
            crate::ElixirRoot::new(state, runtime_for_view, 7, false, false)
        });
        cx.draw(
            point(px(0.0), px(0.0)),
            size(px(400.0), px(300.0)),
            |_window, _cx| view.clone().into_any_element(),
        );
        (runtime, cx)
    }

    fn redraw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    fn press(cx: &mut VisualTestContext) {
        cx.simulate_click(point(px(10.0), px(10.0)), Modifiers::default());
        redraw(cx);
    }

    // The panel opens 2 pixels below the 20 pixel button; rows sit below its
    // border and padding, 22 pixels apart.
    fn item(index: usize) -> gpui::Point<gpui::Pixels> {
        point(px(60.0), px(20.0 + 2.0 + 5.0 + 11.0 + 22.0 * index as f32))
    }

    fn picked(runtime: &crate::SharedRuntime) -> Vec<String> {
        runtime
            .events
            .lock()
            .unwrap()
            .iter()
            .filter_map(|event| match event {
                crate::NativeEvent::Input {
                    event,
                    value: Some(crate::EventValue::String(value)),
                    ..
                } if event == "picked" => Some(value.clone()),
                _ => None,
            })
            .collect()
    }

    // A row that reports clicks, holding a button shown only while it's hovered.
    fn row_with_hidden_button() -> crate::ElementNode {
        let crate::ElementNode::MenuButtonComponent(mut more) = button() else {
            unreachable!("button builds a menu button");
        };
        more.style.visibility = Some("hidden".to_string());
        more.style.group_hover_visible = Some("row".to_string());

        let mut style = crate::StyleAttrs::default();
        style.group = Some("row".to_string());
        style.width = Some(px(200.0).into());
        style.height = Some(px(20.0).into());
        crate::ElementNode::Div(crate::ContainerNode {
            tag: crate::GeneratedElementTag::Div,
            style,
            id: Some("row".to_string()),
            accessibility: crate::AccessibilitySemantics::default(),
            children: vec![crate::ElementNode::MenuButtonComponent(more)],
            click: Some("row-click".to_string()),
            bounds_change: None,
            focus_request: 0,
            focus: None,
            blur: None,
            motion_request: 0,
            motion_duration: 180,
            motion_delay: 0,
            motion_easing: "ease_out".to_string(),
            motion_policy: "respect_system".to_string(),
            motion_from_opacity: 1.0,
            motion_from_x: 0.0,
            motion_from_y: 0.0,
            window_control: None,
        })
    }

    fn row_clicks(runtime: &crate::SharedRuntime) -> usize {
        runtime
            .events
            .lock()
            .unwrap()
            .iter()
            .filter(|event| format!("{event:?}").contains("row-click"))
            .count()
    }

    #[gpui::test]
    fn a_hidden_button_works_once_its_group_is_hovered(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        let runtime = std::sync::Arc::new(crate::runtime::RuntimeState::new());
        let state = std::sync::Arc::new(crate::WindowState::new(
            row_with_hidden_button(),
            Vec::new(),
        ));
        let runtime_for_view = runtime.clone();
        let (view, cx) = cx.add_window_view(move |_window, _cx| {
            crate::ElixirRoot::new(state, runtime_for_view, 7, false, false)
        });
        cx.draw(
            point(px(0.0), px(0.0)),
            size(px(400.0), px(300.0)),
            |_window, _cx| view.clone().into_any_element(),
        );

        // Before the row is hovered the button isn't there: the row gets the click.
        cx.simulate_mouse_move(point(px(300.0), px(200.0)), None, Modifiers::default());
        redraw(cx);
        press(cx);
        assert_eq!(row_clicks(&runtime), 1);

        // Now the pointer is over the row, so the button shows and keeps its press.
        press(cx);
        assert_eq!(row_clicks(&runtime), 1);
        cx.simulate_click(item(1), Modifiers::default());
        assert_eq!(picked(&runtime), vec!["remove"]);
    }

    #[gpui::test]
    fn pressing_opens_the_menu_below_and_a_click_chooses(cx: &mut gpui::TestAppContext) {
        let (runtime, cx) = render(cx);
        press(cx);
        cx.simulate_click(item(1), Modifiers::default());
        assert_eq!(picked(&runtime), vec!["remove"]);
    }

    #[gpui::test]
    fn pressing_again_closes_the_menu(cx: &mut gpui::TestAppContext) {
        let (runtime, cx) = render(cx);
        press(cx);
        press(cx);
        cx.simulate_click(item(0), Modifiers::default());
        assert!(picked(&runtime).is_empty());

        // The next press opens it again.
        press(cx);
        cx.simulate_click(item(0), Modifiers::default());
        assert_eq!(picked(&runtime), vec!["rename"]);
    }
}
