//! Test to verify that the `transmute_copy` in the `same_state` doesn't do
//! any unsound things, because at one point I had the following implementation:
//!
//! ```rust
//! fn same_state(
//!     lhs: &<M as StateMachine>::Superstate<'_>,
//!     rhs: &<M as StateMachine>::Superstate<'_>,
//! ) -> bool {
//!     use core::mem::{discriminant, transmute_copy, Discriminant};
//!     
//!     // Generic associated types are invariant over any lifetime arguments, so the
//!     // compiler won't allow us to compare them directly. Instead we need to coerce them
//!     // to have the same lifetime by transmuting them to the same type.
//!     
//!     let lhs: Discriminant<&<M as StateMachine>::Superstate<'_>> =
//!         unsafe { transmute_copy(&discriminant(lhs)) };
//!     let rhs: Discriminant<&<M as StateMachine>::Superstate<'_>> =
//!         unsafe { transmute_copy(&discriminant(rhs)) };
//!     
//!     lhs == rhs
//! }
//! ```
//!
//! The thing that is wrong here, is that the transmute_copy retuns a
//! `Discriminant<&<M as StateMachine>::Superstate<'_>> where the discriminant
//! is taken from the *reference* of the enum type. This is wrong as the
//! discriminant should be taken from the enum itself or in this case:
//! `Discriminant<<M as StateMachine>::Superstate<'_>>`. But as the discriminant
//! of anything else then an enum is unspecified, this doesn't necessarily cause
//! a bug when the state machine is excecuting. I seems that taking the discriminant
//! of anything else then a enum just uses the first byte in memory as the
//! and as long there are less then 256 superstates, there won't be any difference
//! between the discriminant of a enum or the discriminant of a reference to
//! that same enum.
//!
//! This test would explicitly trigger the bug by forcing the superstate enum to have
//! a discriminant larger then one byte, causing two different superstates to be
//! considered the same.
#[cfg(test)]
mod test {
    use statig::prelude::*;

    struct Event;

    struct Foo;

    enum State {
        S11 = 1,
    }

    enum Superstate {
        S1 = 1,
        S2 = 257,
    }

    impl StateMachine for Foo {
        type Event = Event;

        type Context = Self;

        type State = State;

        type Superstate<'a> = Superstate;

        const INIT_STATE: Self::State = State::S11;
    }

    impl statig::State<Foo> for State {
        fn call_handler(&mut self, _context: &mut Foo, _event: &Event) -> Response<Self> {
            Handled
        }
    }

    impl statig::Superstate<Foo> for Superstate {
        fn call_handler(&mut self, _context: &mut Foo, _event: &Event) -> Response<State> {
            Handled
        }
    }

    #[test]
    fn main() {
        assert!(!Superstate::same_state(&Superstate::S1, &Superstate::S2));
    }
}
