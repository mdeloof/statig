#![deny(missing_docs)]
//! Critical validation test: This MUST fail to prove our doc comment tests work.
//!
//! This test intentionally has missing doc comments on a state handler.
//! The #![deny(missing_docs)] lint should catch this and cause compilation to fail.
//!
//! If this test passes, it means either:
//! 1. Our doc comment implementation is broken, OR  
//! 2. The missing_docs lint isn't working
//!
//! Either case would invalidate our positive doc comment tests.

use statig::prelude::*;

/// Test machine for missing docs validation
#[derive(Default)]
pub struct ValidationMachine {}

/// State machine to validate missing docs detection
#[state_machine(initial = "State::documented()")]
impl ValidationMachine {
    /// This state has documentation.
    #[state]
    fn documented(&mut self) -> Outcome<State> {
        Transition(State::undocumented())
    }

    // INTENTIONALLY MISSING DOC COMMENT - this should cause compile failure
    #[state]
    fn undocumented(&mut self) -> Outcome<State> {
        Transition(State::documented())
    }

    /// This superstate has doc comments.
    #[superstate]
    fn container(&mut self) -> Outcome<State> {
        Super
    }
}

/// Main function for the test
fn main() {
    let machine = ValidationMachine::default();
    let _state_machine = machine.uninitialized_state_machine().init();
}
