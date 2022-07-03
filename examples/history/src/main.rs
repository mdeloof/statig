#![feature(generic_associated_types)]
#![allow(unused)]

use stateful::state_machine;
use stateful::Response::*;
use stateful::Result;
use stateful::StateMachine;
use stateful::{ResultExt, Stateful};
use std::io::Write;

pub enum Event {
    StartProgram,
    DoorOpened,
    DoorClosed,
    TimerElapsed,
}

pub struct Dishwasher {
    previous_state: State,
}

impl Stateful for Dishwasher {
    type State = State;

    type Input = Event;

    const INIT_STATE: State = State::idle();

    // On every transition we update the previous state, so we can
    // transition back to it.
    fn on_transition(&mut self, source: &Self::State, _: &Self::State) {
        self.previous_state = source.clone();
    }
}

#[state_machine]
#[state(derive(Clone, Debug))]
impl Dishwasher {
    #[state]
    fn idle(input: &Event) -> Result<State> {
        match input {
            Event::StartProgram => Ok(Transition(State::soap())),
            _ => Ok(Super),
        }
    }

    #[state(superstate = "door_closed")]
    fn soap(input: &Event) -> Result<State> {
        match input {
            Event::TimerElapsed => Ok(Transition(State::rinse())),
            _ => Ok(Super),
        }
    }

    #[state(superstate = "door_closed")]
    fn rinse(input: &Event) -> Result<State> {
        match input {
            Event::TimerElapsed => Ok(Transition(State::dry())),
            _ => Ok(Super),
        }
    }

    #[state(superstate = "door_closed")]
    fn dry(input: &Event) -> Result<State> {
        match input {
            Event::TimerElapsed => Ok(Transition(State::idle())),
            _ => Ok(Super),
        }
    }

    #[superstate]
    fn door_closed(input: &Event) -> Result<State> {
        match input {
            // When the door is opened the program needs to be paused until
            // the door is closed again.
            Event::DoorOpened => Ok(Transition(State::door_opened())),
            _ => Ok(Super),
        }
    }

    #[state]
    fn door_opened(&mut self, input: &Event) -> Result<State> {
        match input {
            // When the door is closed again, the program is resumed from
            // the previous state.
            Event::DoorClosed => Ok(Transition(self.previous_state.clone())),
            _ => Ok(Super),
        }
    }
}

fn main() {
    let mut state_machine = StateMachine::new(Dishwasher {
        previous_state: Dishwasher::INIT_STATE,
    });

    state_machine.handle(&Event::StartProgram);

    println!("State: {:?}", state_machine.state()); // State: Soap

    state_machine.handle(&Event::TimerElapsed);

    println!("State: {:?}", state_machine.state()); // State: Rinse

    state_machine.handle(&Event::TimerElapsed);

    println!("State: {:?}", state_machine.state()); // State: Dry

    state_machine.handle(&Event::DoorOpened);

    println!("State: {:?}", state_machine.state()); // State: DoorOpened

    state_machine.handle(&Event::DoorClosed);

    println!("State: {:?}", state_machine.state()); // State: Dry

    state_machine.handle(&Event::TimerElapsed);

    println!("State: {:?}", state_machine.state()); // State: Idle
}
