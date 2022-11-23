#![allow(unused)]

use statig::prelude::*;
use statig::StateMachine;
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

#[state_machine(initial = "State::idle()", state(derive(Debug, Clone)))]
impl Dishwasher {
    #[state]
    fn idle(event: &Event) -> Response<State> {
        match event {
            Event::StartProgram => Transition(State::soap()),
            _ => Super,
        }
    }

    #[state]
    fn soap(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::rinse()),
            _ => Super,
        }
    }

    #[state]
    fn rinse(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::dry()),
            _ => Super,
        }
    }

    #[state]
    fn dry(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::idle()),
            _ => Super,
        }
    }

    #[superstate]
    fn door_closed(event: &Event) -> Response<State> {
        match event {
            // When the door is opened the program needs to be paused until
            // the door is closed again.
            Event::DoorOpened => Transition(State::door_opened()),
            _ => Super,
        }
    }

    #[state]
    fn door_opened(&mut self, event: &Event) -> Response<State> {
        match event {
            // When the door is closed again, the program is resumed from
            // the previous state.
            Event::DoorClosed => Transition(self.previous_state.clone()),
            _ => Super,
        }
    }
}

fn main() {
    let mut state_machine = Dishwasher {
        previous_state: Dishwasher::INITIAL,
    }
    .state_machine()
    .init();

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
