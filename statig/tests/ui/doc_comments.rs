#![deny(missing_docs)]
//! Test doc comment propagation to generated State and Superstate enums

use statig::prelude::*;

/// Machine demonstrating doc comment propagation
#[derive(Default)]
pub struct DocMachine {}

/// Test state machine with various doc comment scenarios
#[state_machine(initial = "State::simple_state()")]
impl DocMachine {
    /// Simple state with basic documentation.
    #[state(superstate = "documented_superstate")]
    fn simple_state(&mut self) -> Outcome<State> {
        Transition(State::multi_line_state())
    }

    /// Multi-line state with comprehensive documentation.
    ///
    /// # Purpose
    /// This state demonstrates multi-line doc comments
    /// with various formatting including:
    ///
    /// - Bullet points
    /// - Headers
    /// - Multiple paragraphs
    ///
    /// All of this documentation should be preserved
    /// on the generated State::MultiLineState variant.
    #[state(superstate = "documented_superstate")]
    fn multi_line_state(&mut self) -> Outcome<State> {
        Transition(State::standalone_state())
    }

    /// Standalone state without superstate grouping.
    ///
    /// This tests doc comment propagation for states that
    /// are not part of any superstate hierarchy.
    #[state]
    fn standalone_state(&mut self) -> Outcome<State> {
        Transition(State::simple_state())
    }

    /// Documented superstate containing multiple states.
    ///
    /// This superstate groups together all the documented states
    /// and demonstrates that superstate doc comments are also
    /// properly propagated to the generated Superstate enum.
    #[superstate]
    fn documented_superstate(&mut self) -> Outcome<State> {
        Super
    }
}

/// Main function for the test
fn main() {
    let machine = DocMachine::default();
    let _state_machine = machine.uninitialized_state_machine().init();
}
