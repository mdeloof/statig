use core::fmt::Debug;

/// Holds a reference to either a state or superstate.
pub enum StateOrSuperstate<'a, State, Superstate> {
    /// Reference to a state.
    State(&'a State),
    /// Reference to a superstate.
    Superstate(&'a Superstate),
}

impl<'a, State, Superstate> core::fmt::Debug for StateOrSuperstate<'a, State, Superstate>
where
    State: Debug,
    Superstate: Debug,
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

impl<'a, State, Superstate> PartialEq for StateOrSuperstate<'a, State, Superstate>
where
    State: PartialEq,
    Superstate: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::State(state), Self::State(other)) => state == other,
            (Self::Superstate(superstate), Self::Superstate(other)) => superstate == other,
            _ => false,
        }
    }
}

impl<'a, State, Superstate> Eq for StateOrSuperstate<'a, State, Superstate>
where
    State: Eq,
    Superstate: Eq,
{
}
