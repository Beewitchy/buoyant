//! Focus handling tests for `Rotary`
//!
//! Rotary is a focus-based container that operates in three states:
//! - `UnFocused`: Not focused, passes Next/Previous through to parent
//! - `Focused`: Has focus, Select enters captive mode
//! - `Captive`: Traps focus, Next/Previous fire page events, Blur/Select exits
//!

use buoyant::{
    event::EventResult, focus::Role, match_view, primitives::Size, render::ContentShape,
    view::prelude::*,
};

use buoyant::view::rotary::{Rotary, RotaryEvent, RotaryState};

use crate::focus::harness::FocusTestHarness;

#[derive(Default)]
struct State {
    value: i32,
    focused_count: u32,
    next_count: u32,
    previous_count: u32,
    exit_count: u32,
}

fn rotary_view(_state: &State) -> impl View<(), State> + use<> {
    Rotary::new(
        |state: &mut State, event: &RotaryEvent| match event {
            RotaryEvent::Focused => state.focused_count += 1,
            RotaryEvent::Next => {
                state.next_count += 1;
                state.value += 1;
            }
            RotaryEvent::Previous => {
                state.previous_count += 1;
                state.value -= 1;
            }
            RotaryEvent::Exit => state.exit_count += 1,
        },
        move |rotary_state| {
            match_view!(rotary_state, {
                RotaryState::UnFocused => Circle,  // Unfocused indicator
                RotaryState::Focused => Rectangle, // Focused indicator
                RotaryState::Captive => RoundedRectangle::new(10), // Captive indicator
            })
        },
    )
}

fn rotary_in_stack(_: &State) -> impl View<(), State> + use<> {
    VStack::new((
        Button::new(|_: &mut State| {}, |_| Circle),
        Rotary::new(
            |state: &mut State, event: &RotaryEvent| match event {
                RotaryEvent::Focused => state.focused_count += 1,
                RotaryEvent::Next => {
                    state.next_count += 1;
                    state.value += 1;
                }
                RotaryEvent::Previous => {
                    state.previous_count += 1;
                    state.value -= 1;
                }
                RotaryEvent::Exit => state.exit_count += 1,
            },
            |rotary_state| {
                match_view!(rotary_state, {
                    RotaryState::UnFocused => Circle,
                    RotaryState::Focused => Rectangle,
                    RotaryState::Captive => RoundedRectangle::new(10),
                })
            },
        ),
        Button::new(|_: &mut State| {}, |_| RoundedRectangle::new(5)),
    ))
}

/// Tests the complete state lifecycle of Rotary:
/// `UnFocused` -> `Focused` -> `Captive` -> `Focused` (via blur/select) -> `Captive` (re-entry)
#[test]
fn state_lifecycle_and_transitions() {
    let state = State::default();
    let mut harness =
        FocusTestHarness::new(rotary_view, state, Size::new(100, 100)).with_roles(Role::Button);

    // UnFocused -> Focused: Focus should succeed
    let result = harness.focus_forward();
    assert!(result.has_focus());
    assert_eq!(
        harness.state.focused_count, 0,
        "Focused event only fires on captive entry"
    );

    // Focused -> Captive: Select enters captive mode and fires Focused event
    let result = harness.select();
    assert!(result.has_focus());
    assert_eq!(
        harness.state.focused_count, 1,
        "Should fire Focused event on captive entry"
    );

    // Captive -> Focused: Blur exits captive mode and fires Exit event
    let result = harness.blur();
    assert!(
        result.has_focus(),
        "Should remain focused after blur from captive"
    );
    assert_eq!(harness.state.exit_count, 1, "Should fire Exit event");

    // Focused -> Captive again: Re-enter captive mode
    harness.select();
    assert_eq!(
        harness.state.focused_count, 2,
        "Should fire Focused event again"
    );

    // Make some changes while captive
    harness.next();
    harness.next();
    assert_eq!(harness.state.value, 2);

    // Captive -> Focused: Select also exits captive mode
    let result = harness.select();
    assert!(result.has_focus());
    assert_eq!(
        harness.state.exit_count, 2,
        "Select should also fire Exit event"
    );

    // Re-enter and verify state persists
    harness.select();
    assert_eq!(harness.state.focused_count, 3);
    harness.previous();
    assert_eq!(
        harness.state.value, 1,
        "Value should continue from where we left off"
    );
}

/// Tests that Next/Previous fire events correctly when in captive mode
#[test]
fn captive_mode_navigation_events() {
    let state = State::default();
    let mut harness =
        FocusTestHarness::new(rotary_view, state, Size::new(100, 100)).with_roles(Role::Button);

    harness.focus_forward();
    harness.select(); // Enter captive

    // Next should fire RotaryEvent::Next and increment value
    let result = harness.next();
    assert!(result.has_focus());
    assert_eq!(harness.state.next_count, 1);
    assert_eq!(harness.state.value, 1);

    // Multiple next events
    harness.next();
    harness.next();
    assert_eq!(harness.state.next_count, 3);
    assert_eq!(harness.state.value, 3);

    // Previous should fire RotaryEvent::Previous and decrement value
    let result = harness.previous();
    assert!(result.has_focus());
    assert_eq!(harness.state.previous_count, 1);
    assert_eq!(harness.state.value, 2);

    // Multiple previous events
    harness.previous();
    harness.previous();
    harness.previous();
    harness.previous();
    assert_eq!(harness.state.previous_count, 5);
    assert_eq!(harness.state.value, -2);
}

/// Tests that Next/Previous/Blur defer when not in captive mode
#[test]
fn non_captive_events_defer() {
    let state = State::default();
    let mut harness =
        FocusTestHarness::new(rotary_view, state, Size::new(100, 100)).with_roles(Role::Button);

    harness.focus_forward(); // Focused but not captive

    // Next should defer (pass through to parent)
    let result = harness.next();
    assert!(
        matches!(result, EventResult::Deferred),
        "Next when not captive should defer"
    );
    assert_eq!(harness.state.next_count, 0, "Should not fire Next event");

    // Previous should defer (pass through to parent)
    let result = harness.previous();
    assert!(
        matches!(result, EventResult::Deferred),
        "Previous when not captive should defer"
    );
    assert_eq!(
        harness.state.previous_count, 0,
        "Should not fire Previous event"
    );

    // Blur when not captive should defer
    let result = harness.blur();
    assert!(
        matches!(result, EventResult::Deferred),
        "Blur when not captive should defer"
    );
    assert_eq!(harness.state.exit_count, 0, "Should not fire Exit event");
}

/// Tests navigation behavior when Rotary is inside a `VStack` because `VStack` is weird
#[test]
fn stack_navigation_and_focus_trapping() {
    let state = State::default();
    let mut harness =
        FocusTestHarness::new(rotary_in_stack, state, Size::new(100, 100)).with_roles(Role::Button);

    // Forward navigation through stack: Button1 -> Rotary -> Button2
    let result = harness.focus_forward();
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Should focus first button"
    );

    let result = harness.next();
    assert!(result.has_focus(), "Should focus Rotary");

    // Next from Rotary when not captive should move to next button
    let result = harness.next();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::RoundedRectangle(_))),
        "Should move to third button"
    );

    // Backward navigation: Button2 -> Rotary -> Button1
    let result = harness.previous();
    assert!(result.has_focus(), "Should focus Rotary");

    let result = harness.previous();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Should move to first button"
    );

    // Navigate to Rotary and enter captive mode
    harness.next();
    harness.select();
    assert_eq!(harness.state.focused_count, 1);

    // Captive mode should trap navigation - Next stays in Rotary
    let result = harness.next();
    assert!(result.has_focus());
    assert_eq!(
        harness.state.next_count, 1,
        "Should fire Next event, not navigate"
    );

    // Previous also stays in Rotary
    let result = harness.previous();
    assert!(result.has_focus());
    assert_eq!(
        harness.state.previous_count, 1,
        "Should fire Previous event, not navigate"
    );

    // Exit captive with blur
    harness.blur();
    assert_eq!(harness.state.exit_count, 1);

    // Now Next should move to next button
    let result = harness.next();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::RoundedRectangle(_))),
        "After exiting captive, should move to next button"
    );

    // Test focus_backward from end
    let mut harness2 =
        FocusTestHarness::new(rotary_in_stack, State::default(), Size::new(100, 100))
            .with_roles(Role::Button)
            .with_focus_at_end();

    let result = harness2.focus_backward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::RoundedRectangle(_))),
        "Focus backward should start at last button"
    );

    let result = harness2.previous();
    assert!(
        result.has_focus(),
        "Should focus Rotary when going backward"
    );
}
