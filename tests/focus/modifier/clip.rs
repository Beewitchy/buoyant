//! Focus handling tests for clipped modifier

use buoyant::{focus::Role, primitives::Size, render::ContentShape, view::prelude::*};

use crate::focus::harness::FocusTestHarness;

struct State {
    tapped: bool,
}

#[test]
fn clipped_view_can_receive_focus() {
    fn view(_: &State) -> impl View<(), State> + use<> {
        Button::new(|s: &mut State| s.tapped = true, |_| Circle)
            .frame_sized(100, 100)
            .clipped()
    }

    let state = State { tapped: false };
    let mut harness =
        FocusTestHarness::new(view, state, Size::new(100, 100)).with_roles(Role::Button);

    let result = harness.focus_forward();
    assert!(result.has_focus());
    assert!(
        matches!(result.shape(), Some(ContentShape::Circle(_))),
        "Clipped button should still be focusable"
    );
}
