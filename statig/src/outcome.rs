use core::fmt::Debug;

/// Outcome returned by event handlers in a state machine.
pub enum Outcome<S> {
    /// Consider the event handled.
    Handled,
    /// Defer the event to the superstate.
    Super,
    /// Transition to the given state.
    Transition(S),
}

/// Type alias that will be removed in some future release. Use `Outcome` instead.
#[deprecated(since = "0.4", note = "`Response` has been renamed to `Outcome`")]
pub type Response<S> = Outcome<S>;

impl<S> PartialEq for Outcome<S>
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

impl<S> Eq for Outcome<S> where S: Eq {}

impl<S> Debug for Outcome<S>
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
