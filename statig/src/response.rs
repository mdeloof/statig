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
