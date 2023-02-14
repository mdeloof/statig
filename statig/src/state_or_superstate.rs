use core::fmt::Debug;

use crate::IntoStateMachine;

/// Holds a reference to either a state or superstate.
pub enum StateOrSuperstate<'a, 'b, M: IntoStateMachine>
where
    M::State: 'b,
{
    /// Reference to a state.
    State(&'a M::State),
    /// Reference to a superstate.
    Superstate(&'a M::Superstate<'b>),
}

impl<'a, 'b, M: IntoStateMachine> core::fmt::Debug for StateOrSuperstate<'a, 'b, M>
where
    M::State: Debug,
    M::Superstate<'b>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::State(state) => f.debug_tuple("State").field(state as &dyn Debug).finish(),
            Self::Superstate(superstate) => f
                .debug_tuple("Superstate")
                .field(superstate as &dyn Debug)
                .finish(),
        }
    }
}

impl<'a, 'b, M> PartialEq for StateOrSuperstate<'a, 'b, M>
where
    M: IntoStateMachine,
    M::State: 'b + PartialEq,
    M::Superstate<'b>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::State(state), Self::State(other)) => state == other,
            (Self::Superstate(superstate), Self::Superstate(other)) => superstate == other,
            _ => false,
        }
    }
}

impl<'a, 'b, M> Eq for StateOrSuperstate<'a, 'b, M>
where
    M: IntoStateMachine + PartialEq + Eq,
    M::State: 'b + PartialEq + Eq,
    M::Superstate<'b>: PartialEq + Eq,
{
}
