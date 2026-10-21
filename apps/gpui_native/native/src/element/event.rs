use crate::*;

#[cfg(all(feature = "real-gpui", feature = "components"))]
pub(crate) fn attach_accessible_action_target(
    element: gpui::Stateful<gpui::Div>,
    test_id: String,
    event: String,
    accessibility: super::AccessibilitySemantics,
    runtime: SharedRuntime,
    window_id: u64,
    focus: gpui::FocusHandle,
) -> gpui::Stateful<gpui::Div> {
    use gpui::{AccessibleAction, InteractiveElement, StatefulInteractiveElement};

    let enabled = !accessibility.disabled;
    let mut element = super::apply_accessibility_semantics(element, accessibility);
    element = element.debug_selector(|| test_id.clone());
    let focus = focus.tab_stop(enabled);
    if enabled {
        if let Ok(mut handles) = runtime.focus_handles.lock() {
            handles.insert((window_id, test_id), focus.clone());
        }
        element = element.track_focus(&focus);

        let keyboard_runtime = runtime.clone();
        let keyboard_event = event.clone();
        element = element
            .key_context("GPUIAccessibleControl")
            .tab_index(0)
            .focus_visible(|style| style.border_2().border_color(gpui::rgb(0x60a5fa)))
            .on_key_down(move |key, _window, cx| {
                if matches!(key.keystroke.key.as_str(), "enter" | "space") {
                    emit_click_event(&keyboard_runtime, window_id, &keyboard_event);
                    cx.stop_propagation();
                }
            })
            .on_a11y_action(AccessibleAction::Click, move |_data, _window, _cx| {
                emit_click_event(&runtime, window_id, &event);
            });
    }

    element
}

#[cfg(feature = "real-gpui")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_click_event(
    element: gpui::Div,
    element_id: String,
    event: Option<String>,
    accessibility: super::AccessibilitySemantics,
    runtime: SharedRuntime,
    window_id: u64,
    motion: super::ContainerMotion,
    window_control: Option<String>,
) -> gpui::AnyElement {
    use gpui::{AccessibleAction, InteractiveElement, StatefulInteractiveElement};

    let debug_selector = element_id.clone();
    let element_id = if event.is_some() {
        format!("gpui-elixir-click-{window_id}-{element_id}")
    } else {
        element_id
    };
    let element = element.debug_selector(|| debug_selector).id(element_id);
    let element = super::apply_window_control(element, window_control, runtime.clone(), window_id);

    if let Some(event) = event {
        let enabled = !accessibility.disabled;
        let activatable = enabled
            && accessibility
                .role
                .as_ref()
                .is_some_and(AccessibilityRole::is_activatable);
        let element = super::apply_accessibility_semantics(element, accessibility.clone());
        let element = if activatable {
            let keyboard_runtime = runtime.clone();
            let keyboard_event = event.clone();
            element
                .key_context("GPUIAccessibleControl")
                .tab_index(0)
                .focus_visible(|style| style.border_2().border_color(gpui::rgb(0x60a5fa)))
                .on_key_down(move |key, _window, cx| {
                    if matches!(key.keystroke.key.as_str(), "enter" | "space") {
                        emit_click_event(&keyboard_runtime, window_id, &keyboard_event);
                        cx.stop_propagation();
                    }
                })
        } else {
            element
        };
        let pointer_runtime = runtime.clone();
        let pointer_event = event.clone();

        let element = if enabled {
            element
                // Like phx-click in LiveView, only the innermost clickable
                // element under the pointer handles the click.
                .on_click(move |_event, _window, cx| {
                    emit_click_event(&pointer_runtime, window_id, &pointer_event);
                    cx.stop_propagation();
                })
                .on_a11y_action(AccessibleAction::Click, move |_data, _window, _cx| {
                    emit_click_event(&runtime, window_id, &event);
                })
        } else {
            element
        };

        let actions = if enabled {
            vec![AccessibleAction::Click]
        } else {
            vec![]
        };
        super::apply_container_motion(element, motion, accessibility, actions)
    } else {
        super::apply_container_motion(element, motion, accessibility, vec![])
    }
}

#[cfg(feature = "real-gpui")]
pub(crate) fn emit_click_event(runtime: &SharedRuntime, window_id: u64, event: &str) {
    let _ = push_event(
        runtime,
        NativeEvent::Click {
            window_id,
            event: event.to_string(),
        },
    );
}

#[cfg(feature = "real-gpui")]
pub(crate) fn apply_input_events(
    element: gpui::Div,
    input_id: String,
    keydown: Option<String>,
    keyup: Option<String>,
    runtime: SharedRuntime,
    window_id: u64,
) -> gpui::AnyElement {
    use gpui::{InteractiveElement, IntoElement};

    let mut element = element.key_context("GPUIInput").tab_index(0).id(input_id);

    if let Some(event) = keydown {
        let runtime = runtime.clone();
        element = element.on_key_down(move |key_event, _window, _cx| {
            let value = key_event
                .keystroke
                .key_char
                .clone()
                .or_else(|| Some(key_event.keystroke.key.clone()))
                .map(EventValue::String);

            let _ = push_event(
                &runtime,
                NativeEvent::Input {
                    kind: InputKind::KeyDown,
                    window_id,
                    event: event.clone(),
                    value,
                },
            );
        });
    }

    if let Some(event) = keyup {
        element = element.on_key_up(move |key_event, _window, _cx| {
            let value = key_event
                .keystroke
                .key_char
                .clone()
                .or_else(|| Some(key_event.keystroke.key.clone()))
                .map(EventValue::String);

            let _ = push_event(
                &runtime,
                NativeEvent::Input {
                    kind: InputKind::KeyUp,
                    window_id,
                    event: event.clone(),
                    value,
                },
            );
        });
    }

    element.into_any_element()
}

#[cfg(all(test, feature = "real-gpui"))]
mod tests {
    use super::emit_click_event;
    use crate::gpui;
    use crate::{event::NativeEvent, runtime::RuntimeState, AccessibilityRole};
    use std::sync::Arc;

    #[test]
    fn accessible_action_attachment_uses_the_shared_click_event_path() {
        let runtime = Arc::new(RuntimeState::new());
        emit_click_event(&runtime, 9, "open-settings");

        let events = runtime.events.lock().expect("event queue");
        assert!(matches!(
            events.as_slice(),
            [NativeEvent::Click { window_id: 9, event }] if event == "open-settings"
        ));
    }

    #[test]
    fn generated_role_policy_only_tab_stops_simple_activation_controls() {
        for role in [
            AccessibilityRole::Button,
            AccessibilityRole::Checkbox,
            AccessibilityRole::Link,
            AccessibilityRole::Radio,
            AccessibilityRole::Switch,
        ] {
            assert!(role.is_activatable());
        }

        for role in [
            AccessibilityRole::Group,
            AccessibilityRole::List,
            AccessibilityRole::TabList,
            AccessibilityRole::Tree,
            AccessibilityRole::Slider,
        ] {
            assert!(!role.is_activatable());
        }
    }

    fn clickable(
        id: &str,
        width: f32,
        children: Vec<crate::ElementNode>,
    ) -> crate::ElementNode {
        let mut style = crate::StyleAttrs::default();
        style.width = Some(gpui::px(width).into());
        style.height = Some(gpui::px(20.0).into());
        crate::ElementNode::Div(crate::ContainerNode {
            tag: crate::GeneratedElementTag::Div,
            style,
            id: Some(id.to_string()),
            accessibility: crate::AccessibilitySemantics::default(),
            children,
            click: Some(id.to_string()),
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

    #[gpui::test]
    fn only_the_innermost_clickable_element_handles_a_click(cx: &mut gpui::TestAppContext) {
        use gpui::{point, px, size, IntoElement, Modifiers};

        let tree = clickable("outer", 200.0, vec![clickable("inner", 50.0, Vec::new())]);
        let runtime = Arc::new(RuntimeState::new());
        let state = Arc::new(crate::WindowState::new(tree, Vec::new()));
        let runtime_for_view = runtime.clone();
        let (view, cx) = cx.add_window_view(move |_window, _cx| {
            crate::ElixirRoot::new(state, runtime_for_view, 7, false, false)
        });
        cx.draw(
            point(px(0.0), px(0.0)),
            size(px(300.0), px(100.0)),
            |_window, _cx| view.clone().into_any_element(),
        );

        cx.simulate_click(point(px(10.0), px(10.0)), Modifiers::default());
        cx.simulate_click(point(px(100.0), px(10.0)), Modifiers::default());

        let clicked = runtime
            .events
            .lock()
            .expect("event queue")
            .iter()
            .filter_map(|event| match event {
                NativeEvent::Click { event, .. } => Some(event.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(clicked, vec!["inner", "outer"]);
    }

    #[test]
    fn pointer_and_accessible_activation_share_the_same_native_event_path() {
        let runtime = Arc::new(RuntimeState::new());

        emit_click_event(&runtime, 7, "toggle");
        emit_click_event(&runtime, 7, "toggle");

        let events = runtime.events.lock().expect("event queue");
        assert_eq!(events.len(), 2);
        for event in events.iter() {
            assert!(matches!(
                event,
                NativeEvent::Click { window_id: 7, event } if event == "toggle"
            ));
        }
    }
}
