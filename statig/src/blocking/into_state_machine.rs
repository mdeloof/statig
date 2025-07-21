use crate::StateOrSuperstate;

/// Trait for transforming a type into a state machine.
pub trait IntoStateMachine
where
    Self: Sized,
{
    /// Event that is processed by the state machine.
    type Event<'evt>;

    /// External context that can be passed in.
    type Context<'ctx>;

    /// Response type returned when events are handled.
    type Response: Default;

    /// Enumeration of the various states.
    type State;

    /// Enumeration of the various superstates.
    type Superstate<'sub>
    where
        Self::State: 'sub;

    /// Constructor for the initial state of the state machine.
    fn initial() -> Self::State;

    /// Method that is called *before* an event is dispatched to a state or
    /// superstate handler.
    fn before_dispatch(
        &mut self,
        _state_or_superstate: StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
        _event: &Self::Event<'_>,
        _context: &mut Self::Context<'_>,
    ) {
    }

    /// Method that is called *after* an event is dispatched to a state or
    /// superstate handler.
    fn after_dispatch(
        &mut self,
        _state_or_superstate: StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
        _event: &Self::Event<'_>,
        _context: &mut Self::Context<'_>,
    ) {
    }

    /// Method that is called *before* every transition.
    fn before_transition(
        &mut self,
        _source: &Self::State,
        _target: &Self::State,
        _context: &mut Self::Context<'_>,
    ) {
    }

    /// Method that is called *after* every transition.
    fn after_transition(
        &mut self,
        _source: &Self::State,
        _target: &Self::State,
        _context: &mut Self::Context<'_>,
    ) {
    }
}

pub trait DispatchHook<T, P>
where
    T: IntoStateMachine,
{
    fn call_dispatch_hook(
        &self,
        shared_storage: &mut T,
        state_or_superstate: StateOrSuperstate<'_, T::State, T::Superstate<'_>>,
        event: &T::Event<'_>,
        context: &mut T::Context<'_>,
    );
}

impl<T, F> DispatchHook<T, (T::Event<'_>,)> for F
where
    T: IntoStateMachine,
    F: Fn(&mut T, StateOrSuperstate<'_, T::State, T::Superstate<'_>>, &T::Event<'_>),
{
    fn call_dispatch_hook(
        &self,
        shared_storage: &mut T,
        state_or_superstate: StateOrSuperstate<'_, T::State, T::Superstate<'_>>,
        event: &T::Event<'_>,
        _context: &mut T::Context<'_>,
    ) {
        (self)(shared_storage, state_or_superstate, event)
    }
}

impl<T, F> DispatchHook<T, (T::Event<'_>, T::Context<'_>)> for F
where
    T: IntoStateMachine,
    F: Fn(
        &mut T,
        StateOrSuperstate<'_, T::State, T::Superstate<'_>>,
        &T::Event<'_>,
        &mut T::Context<'_>,
    ),
{
    fn call_dispatch_hook(
        &self,
        shared_storage: &mut T,
        state_or_superstate: StateOrSuperstate<'_, T::State, T::Superstate<'_>>,
        event: &T::Event<'_>,
        context: &mut T::Context<'_>,
    ) {
        (self)(shared_storage, state_or_superstate, event, context)
    }
}

pub trait TransitionHook<T, P>
where
    T: IntoStateMachine,
{
    fn call_transition_hook(
        &self,
        shared_storage: &mut T,
        source: &T::State,
        target: &T::State,
        context: &mut T::Context<'_>,
    );
}

impl<T, F> TransitionHook<T, ()> for F
where
    T: IntoStateMachine,
    F: Fn(&mut T, &T::State, &T::State),
{
    fn call_transition_hook(
        &self,
        shared_storage: &mut T,
        source: &T::State,
        target: &T::State,
        _context: &mut T::Context<'_>,
    ) {
        (self)(shared_storage, source, target)
    }
}

impl<T, F> TransitionHook<T, (T::Context<'_>,)> for F
where
    T: IntoStateMachine,
    F: Fn(&mut T, &T::State, &T::State, &mut T::Context<'_>),
{
    fn call_transition_hook(
        &self,
        shared_storage: &mut T,
        source: &T::State,
        target: &T::State,
        context: &mut T::Context<'_>,
    ) {
        (self)(shared_storage, source, target, context)
    }
}
