#![feature(generic_associated_types)]
#![allow(unused)]

use stateful::state_machine;
use stateful::Response::*;
use stateful::Result;
use stateful::ResultExt;
use stateful::StateMachine;
use std::io::Write;

#[derive(Default)]
struct Blinky;

// The `stateful` trait needs to be implemented on the type that will
// imlement the state machine.
impl stateful::Stateful for Blinky {
    /// The enum that represents the state.
    type State = StateEnum;

    /// The input type that will be submitted to the state machine.
    type Input = Event;

    /// The initial state of the state machine.
    const INIT_STATE: StateEnum = StateEnum::on(false, 23);
}

struct Event;

#[state_machine]
// You can rename the enum that is derived by the state machine macro as well
// as add traits that will be derived from it.
#[state(name = "StateEnum", derive(Clone, Copy))]
impl Blinky {
    // Every state needs to have a `#[state]` attribute added to it.
    #[state(superstate = "playing")]
    fn on(&mut self, led: &mut bool, counter: &mut isize, input: &Event) -> Result<StateEnum> {
        println!("On");
        Ok(Transition(StateEnum::off(false)))
    }

    #[action]
    fn enter_off(&mut self, led: &mut bool) {
        println!("entered off");
        *led = false;
    }

    #[state(superstate = "playing", entry_action = "enter_off")]
    fn off(&mut self, led: &mut bool, input: &Event) -> Result<StateEnum> {
        println!("Off");
        Ok(Transition(StateEnum::on(true, 34)))
    }

    #[superstate]
    fn playing(&mut self, led: &mut bool) -> Result<StateEnum> {
        Ok(Handled)
    }
}

fn main() {
    let mut state_machine = StateMachine::<Blinky>::default();

    for _ in 0..10 {
        state_machine.handle(&Event {});
    }
}
