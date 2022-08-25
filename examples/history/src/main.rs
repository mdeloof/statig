#![feature(generic_associated_types)]
#![allow(unused)]

use stateful::prelude::*;
use stateful::StateMachine;
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

#[derive(Clone, Debug)]
pub enum State {
    Idle,
    Soap,
    Rinse,
    Dry,
    DoorOpened,
}

pub enum Superstate {
    DoorClosed,
}

impl StateMachine for Dishwasher {
    type State = State;

    type Superstate<'a> = Superstate;

    type Input = Event;

    type Context = Self;

    const INIT_STATE: State = State::Idle;

    // On every transition we update the previous state, so we can
    // transition back to it.
    fn on_transition(dishwasher: &mut Dishwasher, source: &Self::State, _: &Self::State) {
        dishwasher.previous_state = source.clone();
    }
}

impl stateful::State<Dishwasher> for State {
    fn call_handler(&mut self, context: &mut Dishwasher, input: &Event) -> Response<Self> {
        match self {
            State::DoorOpened => Dishwasher::door_opened(context, input),
            State::Dry => Dishwasher::dry(input),
            State::Idle => Dishwasher::idle(input),
            State::Soap => Dishwasher::soap(input),
            State::Rinse => Dishwasher::rinse(input),
        }
    }

    fn superstate(&mut self) -> Option<Superstate> {
        match self {
            State::Dry => Some(Superstate::DoorClosed),
            State::Idle => Some(Superstate::DoorClosed),
            State::Soap => Some(Superstate::DoorClosed),
            State::Rinse => Some(Superstate::DoorClosed),
            State::DoorOpened => None,
        }
    }
}

impl stateful::Superstate<Dishwasher> for Superstate {
    fn call_handler(&mut self, context: &mut Dishwasher, input: &Event) -> Response<State> {
        match self {
            Superstate::DoorClosed => Dishwasher::door_closed(input),
        }
    }
}

impl Dishwasher {
    fn idle(input: &Event) -> Response<State> {
        match input {
            Event::StartProgram => Transition(State::Soap),
            _ => Super,
        }
    }

    fn soap(input: &Event) -> Response<State> {
        match input {
            Event::TimerElapsed => Transition(State::Rinse),
            _ => Super,
        }
    }

    fn rinse(input: &Event) -> Response<State> {
        match input {
            Event::TimerElapsed => Transition(State::Dry),
            _ => Super,
        }
    }

    fn dry(input: &Event) -> Response<State> {
        match input {
            Event::TimerElapsed => Transition(State::Idle),
            _ => Super,
        }
    }

    fn door_closed(input: &Event) -> Response<State> {
        match input {
            // When the door is opened the program needs to be paused until
            // the door is closed again.
            Event::DoorOpened => Transition(State::DoorOpened),
            _ => Super,
        }
    }

    fn door_opened(&mut self, input: &Event) -> Response<State> {
        match input {
            // When the door is closed again, the program is resumed from
            // the previous state.
            Event::DoorClosed => Transition(self.previous_state.clone()),
            _ => Super,
        }
    }
}

fn main() {
    let mut state_machine = Dishwasher::with_context(Dishwasher {
        previous_state: Dishwasher::INIT_STATE,
    })
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
