use buoyant::{
    event::EventResult, focus::Role, primitives::Size, render::ContentShape, view::prelude::*,
};

use crate::focus::harness::FocusTestHarness;

struct State {
    tapped: bool,
}

fn single_button_view(_: &State) -> impl View<(), State> + use<> {
    Button::new(|s: &mut State| s.tapped = true, |_| Circle)
}

#[test]
fn single_button_focus() {
    let state = State { tapped: false };
    let mut harness = FocusTestHarness::new(single_button_view, state, Size::new(100, 100))
        .with_roles(Role::Button);

    let result = harness.focus_forward();
    assert!(result.has_focus(), "Single button should be focusable");
    assert!(matches!(result.shape(), Some(ContentShape::Circle(_))));
}

#[test]
fn single_button_next_returns_deferred() {
    let state = State { tapped: false };
    let mut harness = FocusTestHarness::new(single_button_view, state, Size::new(100, 100))
        .with_roles(Role::Button);

    // Focus the button
    harness.focus_forward();

    // Next on single element should return Deferred
    let result = harness.next();
    assert!(matches!(result, EventResult::Deferred));
}

#[test]
fn select_triggers_action() {
    let state = State { tapped: false };
    let mut harness = FocusTestHarness::new(single_button_view, state, Size::new(100, 100))
        .with_roles(Role::Button);

    harness.focus_forward();
    assert!(!harness.state.tapped);

    harness.select();
    assert!(harness.state.tapped, "Select should trigger button action");
}
