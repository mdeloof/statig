use core::fmt::Debug;

/// Outcome returned by event handlers in a state machine.
pub enum Outcome<S, R = ()> {
    /// Consider the event handled.
    Handled(R),
    /// Defer the event to the superstate.
    Super,
    /// Transition to the given state.
    Transition(S),
}

/// Type alias that will be removed in some future release. Use `Outcome` instead.
#[deprecated(since = "0.4", note = "`Response` has been renamed to `Outcome`")]
pub type Response<S, R = ()> = Outcome<S, R>;

impl<S, R> PartialEq for Outcome<S, R>
where
    S: PartialEq,
    R: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Handled(r1), Self::Handled(r2)) => r1 == r2,
            (Self::Super, Self::Super) => true,
            (Self::Transition(s), Self::Transition(o)) => s == o,
            _ => false,
        }
    }
}

impl<S, R> Eq for Outcome<S, R> where S: Eq, R: Eq {}

impl<S, R> Debug for Outcome<S, R>
where
    S: Debug,
    R: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Handled(response) => f
                .debug_tuple("Handled")
                .field(response as &dyn Debug)
                .finish(),
            Self::Super => f.debug_tuple("Super").finish(),
            Self::Transition(state) => f
                .debug_tuple("Transition")
                .field(state as &dyn Debug)
                .finish(),
        }
    }
}
