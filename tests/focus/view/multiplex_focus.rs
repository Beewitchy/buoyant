use buoyant::{
    event::EventResult, focus, primitives::Size, render::ContentShape, view::prelude::*,
};

use crate::focus::harness::FocusTestHarness;

struct State {
    a: u32,
    b: u32,
    c: u32,
}

/// A stack with two buttons, each in a different focus group
fn two_button_stack(_state: &State) -> impl View<(), State> + use<> {
    VStack::new((
        Button::new(|s: &mut State| s.a += 1, |_| Circle).gate_focus(focus::GROUP_0),
        Button::new(|s: &mut State| s.b += 1, |_| Rectangle).gate_focus(focus::GROUP_1),
    ))
    .multiplex_focus::<2>()
}

/// A stack with three buttons, the third being ungated
fn three_button_stack(_state: &State) -> impl View<(), State> + use<> {
    VStack::new((
        Button::new(|s: &mut State| s.a += 1, |_| Circle).gate_focus(focus::GROUP_0),
        Button::new(|s: &mut State| s.b += 1, |_| Rectangle).gate_focus(focus::GROUP_1),
        Button::new(|s: &mut State| s.c += 1, |_| RoundedRectangle::new(10)),
    ))
    .multiplex_focus::<2>()
}

#[test]
fn groups_are_independent() {
    let state = State { a: 0, b: 0, c: 0 };
    let mut harness = FocusTestHarness::new(three_button_stack, state, Size::new(100, 100));

    harness.focus_forward_group(focus::GROUP_0);
    assert!(matches!(
        harness.focus_forward_group(focus::GROUP_0).shape(),
        Some(ContentShape::Circle(_))
    ),);
    assert!(matches!(
        harness.focus_forward_group(focus::GROUP_1).shape(),
        Some(ContentShape::Rectangle(_))
    ),);
    assert!(matches!(
        harness.next_group(focus::GROUP_0).shape(),
        Some(ContentShape::RoundedRectangle(_))
    ),);
    assert!(matches!(
        harness.next_group(focus::GROUP_1).shape(),
        Some(ContentShape::RoundedRectangle(_))
    ),);
    assert!(matches!(
        harness.next_group(focus::GROUP_0),
        EventResult::Deferred
    ));
    assert!(matches!(
        harness.next_group(focus::GROUP_1),
        EventResult::Deferred
    ));
}

#[test]
fn unfocused_behavior() {
    let state = State { a: 0, b: 0, c: 0 };
    let mut harness = FocusTestHarness::new(three_button_stack, state, Size::new(100, 100));

    assert!(harness.focus_forward_group(focus::GROUP_0).has_focus());
    assert!(!harness.blur_group(focus::GROUP_0).has_focus());
    assert!(harness.select_group(focus::GROUP_0).has_focus());

    // Programmer error to blur/select when no focus was obtained
    assert!(matches!(
        harness.blur_group(focus::GROUP_1),
        EventResult::Deferred
    ));
    assert!(matches!(
        harness.select_group(focus::GROUP_1),
        EventResult::Deferred
    ));
}

#[test]
fn select_across_groups() {
    let state = State { a: 0, b: 0, c: 0 };
    let mut harness = FocusTestHarness::new(three_button_stack, state, Size::new(100, 100));

    harness.focus_forward_group(focus::GROUP_0);
    harness.select_group(focus::GROUP_0);
    assert_eq!(harness.state.a, 1);

    harness.focus_forward_group(focus::GROUP_1);
    harness.select_group(focus::GROUP_1);
    assert_eq!(harness.state.b, 1);

    harness.next_group(focus::GROUP_0);
    harness.select_group(focus::GROUP_0);
    assert_eq!(harness.state.c, 1);

    harness.previous_group(focus::GROUP_0);
    harness.select_group(focus::GROUP_0);
    assert_eq!(harness.state.a, 2);
}

#[test]
fn focus_from_opposite_ends() {
    let state = State { a: 0, b: 0, c: 0 };
    let mut harness = FocusTestHarness::new(three_button_stack, state, Size::new(100, 100));

    assert!(matches!(
        harness.focus_forward_group(focus::GROUP_0).shape(),
        Some(ContentShape::Circle(_))
    ),);
    assert!(matches!(
        harness.focus_backward_group(focus::GROUP_1).shape(),
        Some(ContentShape::RoundedRectangle(_))
    ),);
    assert!(matches!(
        harness.next_group(focus::GROUP_0).shape(),
        Some(ContentShape::RoundedRectangle(_))
    ),);
    assert!(matches!(
        harness.previous_group(focus::GROUP_1).shape(),
        Some(ContentShape::Rectangle(_))
    ),);
    assert!(matches!(
        harness.next_group(focus::GROUP_0),
        EventResult::Deferred
    ));
    assert!(matches!(
        harness.previous_group(focus::GROUP_1),
        EventResult::Deferred
    ));
}
