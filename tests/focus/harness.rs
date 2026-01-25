//! Test harness for focus tests to reduce boilerplate

use std::time::Duration;

use buoyant::{
    environment::DefaultEnvironment,
    event::{Event, EventContext, EventResult},
    focus::{DefaultFocus, FocusAction, FocusDirection, FocusGroup, RoleSet},
    primitives::{Point, Size},
    view::ViewLayout,
};
use embedded_touch::{Tool, Touch};

/// A test harness that simplifies focus testing by managing all the boilerplate.
#[allow(dead_code)]
pub struct FocusTestHarness<V, S>
where
    V: ViewLayout<S>,
{
    pub state: S,
    view_fn: fn(&S) -> V,
    view: V,
    view_state: V::State,
    render_tree: V::Renderables,
    focus_tree: V::FocusTree,
    env: DefaultEnvironment,
    event_context: EventContext,
    size: Size,
}

impl<V, S> FocusTestHarness<V, S>
where
    V: ViewLayout<S>,
    V::FocusTree: DefaultFocus,
{
    /// Creates a new test harness with the given view, state, and layout size.
    /// The view function receives a reference to the state, allowing views to
    /// read from state during construction (e.g., for dynamic sizing).
    pub fn new(view_fn: fn(&S) -> V, state: S, size: Size) -> Self
    where
        S: 'static,
    {
        let mut state = state;
        let view = (view_fn)(&state);
        let env = DefaultEnvironment::non_animated();
        let mut view_state = view.build_state(&mut state);
        let layout = view.layout(&size.into(), &env, &mut state, &mut view_state);
        let render_tree = view.render_tree(
            &layout.sublayouts,
            Point::zero(),
            &env,
            &mut state,
            &mut view_state,
        );
        let focus_tree = DefaultFocus::default_first();
        let event_context = EventContext::new(Duration::default());

        Self {
            state,
            view_fn,
            view,
            view_state,
            render_tree,
            focus_tree,
            env,
            event_context,
            size,
        }
    }

    #[must_use]
    pub fn with_roles(mut self, roles: impl Into<RoleSet>) -> Self {
        self.event_context.roles = roles.into();
        self
    }

    /// Initializes the focus tree to the last element (for testing backward navigation).
    #[must_use]
    pub fn with_focus_at_end(mut self) -> Self {
        self.focus_tree = DefaultFocus::default_last();
        self
    }

    /// Updates the size proposed to the view, and performs a view rebuild.
    pub fn resize(&mut self, new_size: Size) {
        self.size = new_size;
        self.rebuild();
    }

    /// Rebuilds the view and render tree with the current size.
    /// Call this after modifying state that affects layout.
    pub fn rebuild(&mut self) {
        self.view = (self.view_fn)(&self.state);
        let layout = self.view.layout(
            &self.size.into(),
            &self.env,
            &mut self.state,
            &mut self.view_state,
        );
        self.render_tree = self.view.render_tree(
            &layout.sublayouts,
            Point::zero(),
            &self.env,
            &mut self.state,
            &mut self.view_state,
        );
    }

    pub fn send_with_group(&mut self, event: impl Into<Event>, group: FocusGroup) -> EventResult {
        let mut context = self.event_context.clone();
        context.focus_group = group;

        let event = event.into();
        let result = self.view.handle_event(
            &event,
            &context,
            &mut self.render_tree,
            &mut self.state,
            &mut self.view_state,
            &mut self.focus_tree,
        );

        if context.view_rebuild_requested.get() {
            self.rebuild();
        }
        result
    }

    /// Sends an event with the default focus group and returns the result.
    ///
    /// Defaults to the common group
    pub fn send(&mut self, event: impl Into<Event>) -> EventResult {
        self.send_with_group(event, FocusGroup::common_group())
    }

    /// Acquires focus searching forward (towards the end).
    pub fn focus_forward(&mut self) -> EventResult {
        self.send(FocusAction::Focus(FocusDirection::Forward))
    }

    /// Acquires focus searching forward in the specified group.
    pub fn focus_forward_group(&mut self, group: FocusGroup) -> EventResult {
        self.send_with_group(FocusAction::Focus(FocusDirection::Forward), group)
    }

    /// Acquires focus searching backward (towards the beginning).
    #[allow(dead_code)]
    pub fn focus_backward(&mut self) -> EventResult {
        self.send(FocusAction::Focus(FocusDirection::Backward))
    }

    /// Acquires focus searching backward in the specified group.
    #[allow(dead_code)]
    pub fn focus_backward_group(&mut self, group: FocusGroup) -> EventResult {
        self.send_with_group(FocusAction::Focus(FocusDirection::Backward), group)
    }

    /// Moves focus to the next element.
    pub fn next(&mut self) -> EventResult {
        self.send(FocusAction::Next)
    }

    /// Moves focus to the next element in the specified group.
    pub fn next_group(&mut self, group: FocusGroup) -> EventResult {
        self.send_with_group(FocusAction::Next, group)
    }

    /// Moves focus to the previous element.
    pub fn previous(&mut self) -> EventResult {
        self.send(FocusAction::Previous)
    }

    /// Moves focus to the previous element in the specified group.
    pub fn previous_group(&mut self, group: FocusGroup) -> EventResult {
        self.send_with_group(FocusAction::Previous, group)
    }

    /// Activates the currently focused element.
    pub fn select(&mut self) -> EventResult {
        self.send(FocusAction::Select)
    }

    /// Activates the currently focused element in the specified group.
    pub fn select_group(&mut self, group: FocusGroup) -> EventResult {
        self.send_with_group(FocusAction::Select, group)
    }

    /// Blurs (exits) the current focus.
    #[allow(dead_code)]
    pub fn blur(&mut self) -> EventResult {
        self.send(FocusAction::Blur)
    }

    /// Blurs (exits) the current focus in the specified group.
    pub fn blur_group(&mut self, group: FocusGroup) -> EventResult {
        self.send_with_group(FocusAction::Blur, group)
    }

    /// Sends a tap (touch down + touch up) at the given point.
    #[allow(dead_code)]
    pub fn tap(&mut self, point: Point) {
        self.send(Event::Touch(Touch::new(
            0,
            point.into(),
            embedded_touch::Phase::Started,
            Tool::Finger,
        )));

        self.send(Event::Touch(Touch::new(
            0,
            point.into(),
            embedded_touch::Phase::Ended,
            Tool::Finger,
        )));
    }
}
