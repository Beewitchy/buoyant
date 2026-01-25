use crate::{
    environment::LayoutEnvironment,
    event::{Event, EventContext, EventResult},
    focus::DefaultFocus,
    layout::ResolvedLayout,
    primitives::{Point, ProposedDimensions},
    render::IntrinsicShape,
    view::{ViewLayout, ViewMarker},
};

#[derive(Clone, Debug)]
pub struct Paginate<V, Action> {
    view: V,
    action: Action,
}

#[derive(Clone, Debug)]
pub enum PageEvent {
    Focused,
    Next,
    Previous,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageState {
    UnFocused,
    Focused,
    Captive,
}

impl<V: ViewMarker, Action> Paginate<V, Action> {
    pub fn new<C>(action: Action, view: V) -> Self
    where
        V: ViewLayout<C>,
        Action: Fn(&mut C, &PageEvent),
    {
        Self { view, action }
    }
}

#[derive(Debug, Clone)]
pub struct PaginateFocusTree<T> {
    is_captive: bool,
    inner: T,
}

impl<T: DefaultFocus> DefaultFocus for PaginateFocusTree<T> {
    fn default_first() -> Self {
        Self {
            is_captive: true,
            inner: T::default_first(),
        }
    }

    fn default_last() -> Self {
        Self {
            is_captive: true,
            inner: T::default_last(),
        }
    }
}

impl<V: ViewMarker, Action> ViewMarker for Paginate<V, Action> {
    type Renderables = V::Renderables;
    type Transition = V::Transition;
}

impl<C, V, Action> ViewLayout<C> for Paginate<V, Action>
where
    V: ViewLayout<C, Renderables: IntrinsicShape>,
    Action: Fn(&mut C, &PageEvent),
{
    type State = V::State;

    type Sublayout = V::Sublayout;

    type FocusTree = PaginateFocusTree<V::FocusTree>;

    fn transition(&self) -> Self::Transition {
        self.view.transition()
    }

    fn build_state(&self, captures: &mut C) -> Self::State {
        self.view.build_state(captures)
    }

    fn layout(
        &self,
        offer: &ProposedDimensions,
        env: &impl LayoutEnvironment,
        captures: &mut C,
        state: &mut Self::State,
    ) -> ResolvedLayout<Self::Sublayout> {
        self.view.layout(offer, env, captures, state)
    }

    fn render_tree(
        &self,
        layout: &Self::Sublayout,
        origin: Point,
        env: &impl LayoutEnvironment,
        captures: &mut C,
        state: &mut Self::State,
    ) -> Self::Renderables {
        self.view.render_tree(layout, origin, env, captures, state)
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
        // TODO: This whole view lol
        // Still trying to figure out the right way to do this

        // For non-focus events, delegate to inner view
        self.view.handle_event(
            event,
            context,
            render_tree,
            captures,
            state,
            &mut focus.inner,
        )
    }
}
