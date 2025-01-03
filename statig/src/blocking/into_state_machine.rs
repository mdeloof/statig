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

    /// Initial state of the state machine.
    const INITIAL: fn() -> Self::State;

    /// Method that is called *before* an event is dispatched to a state or
    /// superstate handler.
    const BEFORE_DISPATCH: fn(
        &mut Self,
        StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
        &Self::Event<'_>,
    ) = |_, _, _| {};

    /// Method that is called *after* an event is dispatched to a state or
    /// superstate handler.
    const AFTER_DISPATCH: fn(
        &mut Self,
        StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
        &Self::Event<'_>,
    ) = |_, _, _| {};

    /// Method that is called *before* every transition.
    const BEFORE_TRANSITION: fn(&mut Self, &Self::State, &Self::State) = |_, _, _| {};

    /// Method that is called *after* every transition.
    const AFTER_TRANSITION: fn(&mut Self, &Self::State, &Self::State) = |_, _, _| {};
}
