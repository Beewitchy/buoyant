//! Focus handling tests for opacity modifier

use buoyant::{focus::Role, primitives::Size, render::ContentShape, view::prelude::*};

use crate::focus::harness::FocusTestHarness;

struct State {
    tapped: bool,
}

/// First button is Circle, second is Rectangle to distinguish via shape
fn view_with_invisible_button(_: &State) -> impl View<(), State> + use<> {
    VStack::new((
        Button::new(|s: &mut State| s.tapped = true, |_| Circle).opacity(0),
        Button::new(|s: &mut State| s.tapped = true, |_| Rectangle),
    ))
}

/// Both buttons visible, first is Circle
fn view_with_visible_buttons(_: &State) -> impl View<(), State> + use<> {
    VStack::new((
        Button::new(|s: &mut State| s.tapped = true, |_| Circle),
        Button::new(|s: &mut State| s.tapped = true, |_| Rectangle),
    ))
}

#[test]
fn opacity_zero_skips_focus() {
    let state = State { tapped: false };
    let mut harness = FocusTestHarness::new(view_with_invisible_button, state, Size::new(100, 100))
        .with_roles(Role::Button);

    // Invisible button (Circle) should be skipped - focus lands on Rectangle
    let result = harness.focus_forward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Rectangle(_))),
        "Should skip invisible Circle button"
    );
}

#[test]
fn opacity_one_allows_focus() {
    let state = State { tapped: false };
    let mut harness = FocusTestHarness::new(view_with_visible_buttons, state, Size::new(100, 100))
        .with_roles(Role::Button);

    // First button (Circle) should be focusable
    let result = harness.focus_forward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Should focus first button (Circle)"
    );
}

#[test]
fn partial_opacity_allows_focus() {
    fn view(_: &State) -> impl View<(), State> + use<> {
        VStack::new((
            Button::new(|s: &mut State| s.tapped = true, |_| Circle).opacity(128),
            Button::new(|s: &mut State| s.tapped = true, |_| Rectangle),
        ))
    }

    let state = State { tapped: false };
    let mut harness =
        FocusTestHarness::new(view, state, Size::new(100, 100)).with_roles(Role::Button);

    // Partial opacity should still allow focus
    let result = harness.focus_forward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Should focus first button with partial opacity"
    );
}
