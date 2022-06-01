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

impl Stateful for Blinky {
    type State = State;

    type Input = Event;

    const INIT_STATE: State = State::on();
}

#[state_machine]
impl Blinky {
    #[state]
    fn on(&mut self, input: &Event) -> Result<State> {
        self.led = false;
        Ok(Transition(State::off()))
    }

    #[state]
    fn off(&mut self, input: &Event) -> Result<State> {
        self.led = true;
        Ok(Transition(State::on()))
    }
}

fn main() {
    let mut state_machine = StateMachine::new(Blinky { led: false });

    state_machine.handle(&Event);
}
