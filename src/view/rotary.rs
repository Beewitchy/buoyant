use core::marker::PhantomData;

use crate::{
    environment::LayoutEnvironment,
    event::{Event, EventContext, EventResult},
    focus::{DefaultFocus, FocusAction},
    layout::ResolvedLayout,
    primitives::{Point, ProposedDimensions},
    render::IntrinsicShape,
    transition::Opacity,
    view::{ViewLayout, ViewMarker},
};

#[derive(Clone, Debug)]
pub struct Rotary<V, ViewFn, Action> {
    _view: PhantomData<V>,
    view_fn: ViewFn,
    action: Action,
}

#[derive(Clone, Debug)]
pub enum RotaryEvent {
    Focused,
    Next,
    Previous,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RotaryState {
    UnFocused,
    Focused,
    Captive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RotaryFocus(bool);

impl RotaryFocus {
    fn is_captive(self) -> bool {
        self.0
    }
}

impl DefaultFocus for RotaryFocus {
    fn default_first() -> Self {
        Self(false)
    }

    fn default_last() -> Self {
        Self(false)
    }
}

impl<V: ViewMarker, ViewFn: Fn(&RotaryState) -> V, Action> Rotary<V, ViewFn, Action> {
    pub fn new<C>(action: Action, view_fn: ViewFn) -> Self
    where
        V: ViewLayout<C>,
        Action: Fn(&mut C, &RotaryEvent),
    {
        Self {
            _view: PhantomData,
            view_fn,
            action,
        }
    }
}

impl<V: ViewMarker, ViewFn, Action> ViewMarker for Rotary<V, ViewFn, Action> {
    type Renderables = V::Renderables;
    type Transition = Opacity;
}

impl<C, V, ViewFn, Action> ViewLayout<C> for Rotary<V, ViewFn, Action>
where
    V: ViewLayout<C, Renderables: IntrinsicShape>,
    ViewFn: Fn(&RotaryState) -> V,
    Action: Fn(&mut C, &RotaryEvent),
{
    // FIXME: Shouldn't have to sync here
    type State = (RotaryState, V::State);

    type Sublayout = V::Sublayout;

    type FocusTree = RotaryFocus;

    fn transition(&self) -> Self::Transition {
        Opacity
    }

    fn build_state(&self, captures: &mut C) -> Self::State {
        let s = RotaryState::UnFocused;
        let view = (self.view_fn)(&s);
        (s, view.build_state(captures))
    }

    fn layout(
        &self,
        offer: &ProposedDimensions,
        env: &impl LayoutEnvironment,
        captures: &mut C,
        state: &mut Self::State,
    ) -> ResolvedLayout<Self::Sublayout> {
        // FIXME: Pass focus in layout to avoid state sync?
        let view = (self.view_fn)(&state.0);
        view.layout(offer, env, captures, &mut state.1)
    }

    fn render_tree(
        &self,
        layout: &Self::Sublayout,
        origin: Point,
        env: &impl LayoutEnvironment,
        captures: &mut C,
        state: &mut Self::State,
    ) -> Self::Renderables {
        // FIXME: Pass focus in render to avoid state sync?
        let view = (self.view_fn)(&state.0);
        view.render_tree(layout, origin, env, captures, &mut state.1)
    }

    fn handle_event(
        &self,
        event: &Event,
        context: &EventContext,
        render_tree: &mut Self::Renderables,
        captures: &mut C,
        state: &mut Self::State,
        focus: &mut Self::FocusTree,
    ) -> EventResult {
        // Handle focus events specially
        if let Event::Focus(focus_event) = event {
            context.request_view_rebuild();
            let focused_shape = render_tree.content_shape();

            return if focus.is_captive() {
                match focus_event {
                    FocusAction::Next => {
                        (self.action)(captures, &RotaryEvent::Next);
                        EventResult::handled_focused(focused_shape)
                    }
                    FocusAction::Previous => {
                        (self.action)(captures, &RotaryEvent::Previous);
                        EventResult::handled_focused(focused_shape)
                    }
                    FocusAction::Focus(_) => {
                        state.0 = RotaryState::Captive;
                        EventResult::handled_focused(focused_shape)
                    }
                    FocusAction::Blur | FocusAction::Select => {
                        // FIXME: Customizable exit on select?
                        (self.action)(captures, &RotaryEvent::Exit);
                        focus.0 = false;
                        state.0 = RotaryState::Focused;
                        context.request_view_rebuild();
                        EventResult::handled_focused(focused_shape)
                    }
                }
            } else {
                match focus_event {
                    FocusAction::Next | FocusAction::Previous | FocusAction::Blur => {
                        state.0 = RotaryState::UnFocused;
                        context.request_view_rebuild();
                        EventResult::Deferred
                    }
                    FocusAction::Focus(_) => {
                        state.0 = RotaryState::Focused;
                        EventResult::handled_focused(focused_shape)
                    }
                    FocusAction::Select => {
                        (self.action)(captures, &RotaryEvent::Focused);
                        focus.0 = true;
                        state.0 = RotaryState::Captive;
                        context.request_view_rebuild();
                        EventResult::handled_focused(focused_shape)
                    }
                }
            };
        }

        // For non-focus events, we could delegate to the inner, or allow dragging to inject
        // events. For now, just defer.
        EventResult::Deferred
    }
}
