use core::future::{self, Future};

use crate::StateOrSuperstate;

/// Trait for transorming a type into a state machine.
pub trait IntoStateMachine
where
    Self: Sized,
{
    /// Event that is processed by the state machine.
    type Event<'evt>;

    /// External context that can be passed in.
    type Context<'ctx>;

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
    ) -> impl Future<Output = ()> {
        future::ready(())
    }

    /// Method that is called *after* an event is dispatched to a state or
    /// superstate handler.
    fn after_dispatch(
        &mut self,
        _state_or_superstate: StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
        _event: &Self::Event<'_>,
    ) -> impl Future<Output = ()> {
        future::ready(())
    }

    /// Method that is called *before* every transition.
    fn before_transition(
        &mut self,
        _source: &Self::State,
        _target: &Self::State,
    ) -> impl Future<Output = ()> {
        future::ready(())
    }

    /// Method that is called *after* every transition.
    fn after_transition(
        &mut self,
        _source: &Self::State,
        _target: &Self::State,
    ) -> impl Future<Output = ()> {
        future::ready(())
    }
}
