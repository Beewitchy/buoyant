//! Focus handling tests for overlay modifier
//!
//! Tests focus behavior with overlay views

use buoyant::{
    event::EventResult, focus::Role, layout::Alignment, primitives::Size, render::ContentShape,
    view::prelude::*,
};

use crate::focus::harness::FocusTestHarness;

struct State {
    foreground_tapped: bool,
    overlay_tapped: bool,
}

#[test]
fn overlay_receives_focus_before_foreground() {
    fn test_view(_: &State) -> impl View<(), State> + use<> {
        Circle.overlay(
            Alignment::Center,
            Button::new(|s: &mut State| s.overlay_tapped = true, |_| Rectangle),
        )
    }

    let state = State {
        foreground_tapped: false,
        overlay_tapped: false,
    };
    let mut harness =
        FocusTestHarness::new(test_view, state, Size::new(100, 100)).with_roles(Role::Button);

    // Overlay should receive focus first due to z-order
    let result = harness.focus_forward();
    assert!(result.has_focus(), "Overlay should receive focus");

    // The overlay contains a Rectangle button
    assert!(
        matches!(result.shape(), Some(ContentShape::Rectangle(_))),
        "Overlay button should be a Rectangle"
    );
}

#[test]
fn foreground_receives_focus_when_overlay_empty() {
    fn test_view(_: &State) -> impl View<(), State> + use<> {
        Button::new(|s: &mut State| s.foreground_tapped = true, |_| Circle)
            .overlay(Alignment::Center, EmptyView)
    }

    let state = State {
        foreground_tapped: false,
        overlay_tapped: false,
    };
    let mut harness =
        FocusTestHarness::new(test_view, state, Size::new(100, 100)).with_roles(Role::Button);

    // Should focus the foreground button when overlay is empty
    let result = harness.focus_forward();
    assert!(
        result.has_focus(),
        "Foreground should receive focus when overlay is empty"
    );

    // The foreground is a Circle button
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Foreground button should be a Circle"
    );
}

#[test]
fn empty_overlay_has_no_focusable_content() {
    fn test_view(_: &State) -> impl View<(), State> + use<> {
        VStack::new((
            Button::new(|s: &mut State| s.foreground_tapped = true, |_| Circle),
            Rectangle,
        ))
        .overlay(Alignment::Center, EmptyView)
    }

    let state = State {
        foreground_tapped: false,
        overlay_tapped: false,
    };
    let mut harness =
        FocusTestHarness::new(test_view, state, Size::new(100, 100)).with_roles(Role::Button);

    // Should skip the empty overlay and focus the Circle button
    let result = harness.focus_forward();
    assert!(
        result.has_focus(),
        "Should focus foreground button through empty overlay"
    );
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Should be the Circle button"
    );
}

/// Both overlay and foreground have focusable buttons
fn view_with_both_focusable(_: &State) -> impl View<(), State> + use<> {
    Button::new(|s: &mut State| s.foreground_tapped = true, |_| Circle).overlay(
        Alignment::Center,
        Button::new(|s: &mut State| s.overlay_tapped = true, |_| Rectangle),
    )
}

#[test]
fn forward_navigates_overlay_to_foreground() {
    let state = State {
        foreground_tapped: false,
        overlay_tapped: false,
    };
    let mut harness = FocusTestHarness::new(view_with_both_focusable, state, Size::new(100, 100))
        .with_roles(Role::Button);

    // Focus forward lands on overlay (Rectangle)
    let result = harness.focus_forward();
    assert!(result.has_focus());
    assert!(matches!(result.shape(), Some(ContentShape::Rectangle(_))));

    // Next moves from overlay to foreground (Circle)
    let result = harness.next();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Next from overlay should move to foreground"
    );
}

#[test]
fn forward_past_foreground_returns_deferred() {
    let state = State {
        foreground_tapped: false,
        overlay_tapped: false,
    };
    let mut harness = FocusTestHarness::new(view_with_both_focusable, state, Size::new(100, 100))
        .with_roles(Role::Button);

    // Focus overlay, then foreground
    harness.focus_forward();
    harness.next();

    // Next from foreground should return Deferred (no wrapping)
    let result = harness.next();
    assert!(
        matches!(result, EventResult::Deferred),
        "Forward past foreground should return Deferred, not wrap"
    );
}

#[test]
fn backward_navigates_foreground_to_overlay() {
    let state = State {
        foreground_tapped: false,
        overlay_tapped: false,
    };
    let mut harness = FocusTestHarness::new(view_with_both_focusable, state, Size::new(100, 100))
        .with_roles(Role::Button)
        .with_focus_at_end(); // Start at foreground (last element)

    // Focus backward acquires focus on foreground (Circle) - the current position
    let result = harness.focus_backward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Focus backward should acquire focus on foreground (current position)"
    );

    // Previous moves from foreground to overlay (Rectangle)
    let result = harness.previous();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Rectangle(_))),
        "Previous from foreground should move to overlay"
    );
}

#[test]
fn backward_past_overlay_returns_deferred() {
    let state = State {
        foreground_tapped: false,
        overlay_tapped: false,
    };
    let mut harness = FocusTestHarness::new(view_with_both_focusable, state, Size::new(100, 100))
        .with_roles(Role::Button);

    // Focus overlay directly
    harness.focus_forward();

    // Previous from overlay should return Deferred (no wrapping)
    let result = harness.previous();
    assert!(
        matches!(result, EventResult::Deferred),
        "Backward past overlay should return Deferred, not wrap"
    );
}

#[test]
fn focus_backward_on_first_element_returns_focused() {
    // When focus tree points to first element (overlay) and it's focusable,
    // Focus(Backward) should acquire focus on it (not search backward)
    let state = State {
        foreground_tapped: false,
        overlay_tapped: false,
    };
    let mut harness = FocusTestHarness::new(view_with_both_focusable, state, Size::new(100, 100))
        .with_roles(Role::Button);
    // Focus tree starts at default_first() which is Overlay

    let result = harness.focus_backward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Rectangle(_))),
        "Focus(Backward) on focusable first element should focus it"
    );
}

#[test]
fn previous_on_first_element_returns_deferred() {
    // When focused on first element (overlay), Previous should return Deferred
    let state = State {
        foreground_tapped: false,
        overlay_tapped: false,
    };
    let mut harness = FocusTestHarness::new(view_with_both_focusable, state, Size::new(100, 100))
        .with_roles(Role::Button);

    // First acquire focus on overlay
    harness.focus_forward();

    // Previous should return Deferred (can't go back from first element)
    let result = harness.previous();
    assert!(
        matches!(result, EventResult::Deferred),
        "Previous on first element should return Deferred"
    );
}

#[test]
fn select_triggers_overlay_action() {
    let state = State {
        foreground_tapped: false,
        overlay_tapped: false,
    };
    let mut harness = FocusTestHarness::new(view_with_both_focusable, state, Size::new(100, 100))
        .with_roles(Role::Button);

    harness.focus_forward();
    harness.select();

    assert!(
        harness.state.overlay_tapped,
        "Select should trigger overlay button"
    );
    assert!(
        !harness.state.foreground_tapped,
        "Foreground should not be triggered"
    );
}

#[test]
fn select_triggers_foreground_action() {
    let state = State {
        foreground_tapped: false,
        overlay_tapped: false,
    };
    let mut harness = FocusTestHarness::new(view_with_both_focusable, state, Size::new(100, 100))
        .with_roles(Role::Button);

    harness.focus_forward();
    harness.next(); // Move to foreground
    harness.select();

    assert!(
        !harness.state.overlay_tapped,
        "Overlay should not be triggered"
    );
    assert!(
        harness.state.foreground_tapped,
        "Select should trigger foreground button"
    );
}
