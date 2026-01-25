use core::{marker::PhantomData, time::Duration};

use crate::{
    animation::Animation,
    event::{Event, EventContext, EventResult},
    focus::{DefaultFocus, FocusAction, FocusDirection},
    layout::ResolvedLayout,
    render::{Animate, TransitionOption},
    view::{ViewLayout, ViewMarker},
};

#[derive(Debug, Clone)]
pub struct Popover<Inner, ViewFn, Overlay, T> {
    inner: Inner,
    behavior: Behavior,
    view_fn: ViewFn,
    _overlay: PhantomData<Overlay>,
    value: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Behavior {
    /// Wrap focus around to the other side when reaching the end
    #[default]
    Wrap,
    /// Stop movement at the ends
    Terminate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusTree<T, U> {
    /// The inner view's focus state (always preserved)
    pub inner: T,
    /// The overlay's focus state (when overlay is active)
    pub overlay: Option<U>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopoverState<T, U> {
    pub inner_state: T,
    pub overlay_state: Option<U>,
}

impl<T: DefaultFocus, U: DefaultFocus> DefaultFocus for FocusTree<T, U> {
    fn default_first() -> Self {
        Self {
            inner: DefaultFocus::default_first(),
            overlay: None,
        }
    }

    fn default_last() -> Self {
        Self {
            inner: DefaultFocus::default_last(),
            overlay: None,
        }
    }
}

impl<Inner, Overlay, ViewFn, T> Popover<Inner, ViewFn, Overlay, T>
where
    Inner: ViewMarker,
    Overlay: ViewMarker,
    ViewFn: for<'b> Fn(&'b T) -> Overlay,
    T: Clone,
{
    #[must_use]
    pub fn new(inner: Inner, value: Option<T>, view_fn: ViewFn) -> Self {
        Self {
            inner,
            behavior: Behavior::default(),
            view_fn,
            _overlay: PhantomData,
            value,
        }
    }
}

impl<Inner: ViewMarker, ViewFn, Overlay: ViewMarker, T> ViewMarker
    for Popover<Inner, ViewFn, Overlay, T>
{
    type Renderables = (
        Inner::Renderables,
        Animate<TransitionOption<Overlay::Renderables, Overlay::Transition>, bool>,
    );

    type Transition = Inner::Transition;
}

impl<Captures, Inner, ViewFn, Overlay, T> ViewLayout<Captures>
    for Popover<Inner, ViewFn, Overlay, T>
where
    Captures: ?Sized,
    Inner: ViewLayout<Captures>,
    Overlay: ViewLayout<Captures>,
    ViewFn: for<'b> Fn(&'b T) -> Overlay,
{
    type State = PopoverState<Inner::State, Overlay::State>;
    type Sublayout = ResolvedLayout<Inner::Sublayout>;
    type FocusTree = FocusTree<Inner::FocusTree, Overlay::FocusTree>;

    fn transition(&self) -> Self::Transition {
        self.inner.transition()
    }

    fn priority(&self) -> i8 {
        self.inner.priority()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn build_state(&self, captures: &mut Captures) -> Self::State {
        Self::State {
            inner_state: self.inner.build_state(captures),
            overlay_state: self.value.as_ref().map(|v| {
                let overlay_view = (self.view_fn)(v);
                overlay_view.build_state(captures)
            }),
        }
    }

    fn layout(
        &self,
        offer: &crate::primitives::ProposedDimensions,
        env: &impl crate::environment::LayoutEnvironment,
        captures: &mut Captures,
        state: &mut Self::State,
    ) -> ResolvedLayout<Self::Sublayout> {
        let inner_layout = self
            .inner
            .layout(offer, env, captures, &mut state.inner_state);

        // Take as much space as possible
        let size = offer.resolve_most_flexible(0, 1);
        ResolvedLayout {
            sublayouts: inner_layout,
            resolved_size: size,
        }
    }

    fn render_tree(
        &self,
        layout: &Self::Sublayout,
        origin: crate::primitives::Point,
        env: &impl crate::environment::LayoutEnvironment,
        captures: &mut Captures,
        state: &mut Self::State,
    ) -> Self::Renderables {
        let inner_tree = self.inner.render_tree(
            &layout.sublayouts,
            origin,
            env,
            captures,
            &mut state.inner_state,
        );
        let overlay_tree = match &self.value {
            Some(v) => {
                let overlay_view = (self.view_fn)(v);
                let overlay_state = state
                    .overlay_state
                    .get_or_insert_with(|| overlay_view.build_state(captures));
                let overlay_layout = overlay_view
                    .layout(&layout.resolved_size.into(), env, captures, overlay_state)
                    .sublayouts;
                TransitionOption::new_some(
                    overlay_view.render_tree(&overlay_layout, origin, env, captures, overlay_state),
                    layout.resolved_size.into(),
                    overlay_view.transition(),
                )
            }
            _ => TransitionOption::None,
        };
        (
            inner_tree,
            Animate::new(
                overlay_tree,
                Animation::ease_out(Duration::from_millis(300)),
                env.app_time(),
                self.value.is_some(),
            ),
        )
    }

    fn handle_event(
        &self,
        event: &Event,
        context: &EventContext,
        render_tree: &mut Self::Renderables,
        captures: &mut Captures,
        state: &mut Self::State,
        focus: &mut Self::FocusTree,
    ) -> EventResult {
        // Handle focus events specially - they need to route through the focus tree
        if let Event::Focus(focus_event) = event {
            if let Some(v) = &self.value {
                // Overlay is active - ensure we have overlay focus state
                let subfocus = focus
                    .overlay
                    .get_or_insert_with(DefaultFocus::default_first);

                let view = (self.view_fn)(v);
                let overlay_state = state
                    .overlay_state
                    .get_or_insert_with(|| view.build_state(captures));

                if let TransitionOption::Some { subtree, .. } = &mut render_tree.1.subtree {
                    let result = view.handle_event(
                        &Event::Focus(*focus_event),
                        context,
                        subtree,
                        captures,
                        overlay_state,
                        subfocus,
                    );

                    if matches!(result, EventResult::Deferred) {
                        // Determine if we were moving forward or backward
                        let is_forward = matches!(
                            focus_event,
                            FocusAction::Next | FocusAction::Focus(FocusDirection::Forward)
                        );

                        match self.behavior {
                            Behavior::Wrap => {
                                // Wrap to the opposite end based on direction
                                // Reset focus tree to the appropriate end
                                *subfocus = if is_forward {
                                    DefaultFocus::default_first()
                                } else {
                                    DefaultFocus::default_last()
                                };
                                // Acquire focus at the wrapped position (don't navigate again)
                                let acquire_direction = if is_forward {
                                    FocusDirection::Forward
                                } else {
                                    FocusDirection::Backward
                                };
                                return view.handle_event(
                                    &Event::Focus(FocusAction::Focus(acquire_direction)),
                                    context,
                                    subtree,
                                    captures,
                                    overlay_state,
                                    subfocus,
                                );
                            }
                            Behavior::Terminate => {
                                // Refocus on the element at the boundary we hit
                                let refocus_direction = if is_forward {
                                    FocusDirection::Backward
                                } else {
                                    FocusDirection::Forward
                                };
                                view.handle_event(
                                    &Event::Focus(FocusAction::Focus(refocus_direction)),
                                    context,
                                    subtree,
                                    captures,
                                    overlay_state,
                                    subfocus,
                                );
                            }
                        }
                    }

                    return result;
                }
                // FIXME: Attempt to recover?
                return EventResult::Deferred;
            }
            // Overlay is not active - clear overlay focus and use inner focus
            focus.overlay = None;

            return self.inner.handle_event(
                &Event::Focus(*focus_event),
                context,
                &mut render_tree.0,
                captures,
                &mut state.inner_state,
                &mut focus.inner,
            );
        }

        // FIXME: State handling?
        match (
            &self.value,
            &mut render_tree.1.subtree,
            &mut state.overlay_state,
        ) {
            (Some(v), TransitionOption::Some { subtree, .. }, Some(s)) => {
                let overlay_view = (self.view_fn)(v);
                // FIXME: This doesn't respect direction on init, is it called?
                overlay_view.handle_event(
                    event,
                    context,
                    subtree,
                    captures,
                    s,
                    focus.overlay.get_or_insert(DefaultFocus::default_first()),
                )
            }
            _ => self.inner.handle_event(
                event,
                context,
                &mut render_tree.0,
                captures,
                &mut state.inner_state,
                &mut focus.inner,
            ),
        }
    }
}
