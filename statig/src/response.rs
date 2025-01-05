use core::fmt::Debug;

/// Response returned by event handlers in a state machine.
pub enum Response<S> {
    /// Consider the event handled.
    Handled,
    /// Defer the event to the superstate.
    Super,
    /// Transition to the given state.
    Transition(S),
}

impl<S> PartialEq for Response<S>
where
    S: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Handled, Self::Handled) => true,
            (Self::Super, Self::Super) => true,
            (Self::Transition(s), Self::Transition(o)) => s == o,
            _ => false,
        }
    }
}

impl<S> Eq for Response<S> where S: Eq {}

impl<S> Debug for Response<S>
where
    S: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Handled => f.debug_tuple("Handled").finish(),
            Self::Super => f.debug_tuple("Super").finish(),
            Self::Transition(state) => f
                .debug_tuple("Transition")
                .field(state as &dyn Debug)
                .finish(),
        }
    }
}
