#![feature(generic_associated_types)]
#![allow(unused)]

use stateful::state_machine;
use stateful::Response::*;
use stateful::Result;
use stateful::ResultExt;
use stateful::{StateMachine, Stateful};
use std::io::Write;

struct Blinky {
    led: bool,
}

struct Event;

// The `stateful` trait needs to be implemented on the type that will be
// the context for the state machine.
impl Stateful for Blinky {
    /// The enum that represents the state. This type is derived by the
    /// `#[state_machine]` macro.
    type State = State;

    /// The input type that will be submitted to the state machine.
    type Input = Event;

    /// The initial state of the state machine.
    const INIT_STATE: State = State::on();
}

#[state_machine]
impl Blinky {
    #[state]
    fn on(&mut self, input: &Event) -> Result<State> {
        self.led = false;
        // Transition to the `off` state.
        Ok(Transition(State::off()))
    }

    #[state]
    fn off(&mut self, input: &Event) -> Result<State> {
        self.led = true;
        // Transition to the `on` state.
        Ok(Transition(State::on()))
    }
}

fn main() {
    let mut state_machine = StateMachine::new(Blinky { led: false });

    state_machine.handle(&Event);
}
