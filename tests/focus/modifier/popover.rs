//! Focus handling tests for popover modifier
//!
//! Tests focus behavior when popovers are shown/hidden

use buoyant::{focus::Role, primitives::Size, render::ContentShape, view::prelude::*};

use crate::focus::harness::FocusTestHarness;

/// Creates a popover with multiple focusable elements to test wrapping
fn test_view_with_multiple_focusable(_: &State) -> impl View<(), State> + use<> {
    VStack::new((Button::new(
        |s: &mut State| s.main_tapped = true,
        |_| Rectangle,
    ),))
    .popover(Some(()), |()| {
        // Two buttons in the popover to test navigation between them
        VStack::new((
            Button::new(|_: &mut State| {}, |_| Circle).frame_sized(50, 50),
            Button::new(|_: &mut State| {}, |_| RoundedRectangle::new(5)).frame_sized(50, 50),
        ))
    })
}

#[derive(Clone)]
struct State {
    main_tapped: bool,
    popover_tapped: bool,
}

/// Popover button uses Circle shape to distinguish from main view's Rectangle
fn test_view_with_popover(_: &State) -> impl View<(), State> + use<> {
    VStack::new((Button::new(
        |s: &mut State| s.main_tapped = true,
        |_| Rectangle,
    ),))
    .popover(Some(()), |()| {
        Button::new(|s: &mut State| s.popover_tapped = true, |_| Circle).frame_sized(50, 50)
    })
}

/// No popover shown - just the main button with Rectangle shape
fn test_view_without_popover(_: &State) -> impl View<(), State> + use<> {
    VStack::new((Button::new(
        |s: &mut State| s.main_tapped = true,
        |_| Rectangle,
    ),))
    .popover(None::<()>, |()| {
        Button::new(|s: &mut State| s.popover_tapped = true, |_| Circle)
    })
}

#[test]
fn popover_shown_receives_focus() {
    let state = State {
        main_tapped: false,
        popover_tapped: false,
    };
    let mut harness = FocusTestHarness::new(test_view_with_popover, state, Size::new(100, 100))
        .with_roles(Role::Button);

    // Popover button (Circle) should receive focus first
    let result = harness.focus_forward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Popover content (Circle) should receive focus"
    );
}

#[test]
fn popover_hidden_shows_main_view() {
    let state = State {
        main_tapped: false,
        popover_tapped: false,
    };
    let mut harness = FocusTestHarness::new(test_view_without_popover, state, Size::new(100, 100))
        .with_roles(Role::Button);

    // Without popover, should focus main button (Rectangle)
    let result = harness.focus_forward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Rectangle(_))),
        "Main button (Rectangle) should be focusable when popover is hidden"
    );
}

#[test]
fn popover_wraps_focus_forward() {
    let state = State {
        main_tapped: false,
        popover_tapped: false,
    };
    let mut harness = FocusTestHarness::new(
        test_view_with_multiple_focusable,
        state,
        Size::new(100, 100),
    )
    .with_roles(Role::Button);

    // Focus first element (Circle)
    let result = harness.focus_forward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "First element should be Circle"
    );

    // Navigate to second element (RoundedRectangle) using next()
    let result = harness.next();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::RoundedRectangle(_))),
        "Second element should be RoundedRectangle, got {:?}",
        result.shape()
    );

    // Navigate forward again - should wrap to first element (Circle)
    let result = harness.next();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Should wrap to first element (Circle) when moving forward past end"
    );
}

#[test]
fn popover_wraps_focus_backward_from_start() {
    let state = State {
        main_tapped: false,
        popover_tapped: false,
    };
    let mut harness = FocusTestHarness::new(
        test_view_with_multiple_focusable,
        state,
        Size::new(100, 100),
    )
    .with_roles(Role::Button);

    // First acquire focus on the first element
    let result = harness.focus_forward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "First element should be Circle"
    );

    // Navigate backward from first - should wrap to last element (RoundedRectangle)
    let result = harness.previous();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::RoundedRectangle(_))),
        "Should wrap to last element (RoundedRectangle) when moving backward from start, got {:?}",
        result.shape()
    );
}

#[test]
fn popover_wraps_focus_backward() {
    let state = State {
        main_tapped: false,
        popover_tapped: false,
    };
    let mut harness = FocusTestHarness::new(
        test_view_with_multiple_focusable,
        state,
        Size::new(100, 100),
    )
    .with_roles(Role::Button);

    // Focus first element (Circle)
    let result = harness.focus_forward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "First element should be Circle"
    );

    // Navigate to second element (RoundedRectangle)
    let result = harness.next();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::RoundedRectangle(_))),
        "Second element should be RoundedRectangle"
    );

    // Navigate backward - should go to Circle
    let result = harness.previous();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Should move to previous element (Circle)"
    );

    // Navigate backward again - should wrap to RoundedRectangle
    let result = harness.previous();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::RoundedRectangle(_))),
        "Should wrap to last element (RoundedRectangle) when moving backward past beginning, got {:?}",
        result.shape()
    );
}
