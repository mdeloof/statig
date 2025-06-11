#![allow(unused)]

use statig::blocking::{self, *};
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

impl IntoStateMachine for Dishwasher {
    type State = State;

    type Superstate<'sub> = Superstate;

    type Event<'evt> = Event;

    type Context<'ctx> = ();

    fn initial() -> Self::State {
        State::Idle
    }

    // On every transition we update the previous state, so we can
    // transition back to it.
    fn after_transition(&mut self, source: &Self::State, _target: &Self::State, _context: &mut ()) {
        self.previous_state = source.clone();
    }
}

impl blocking::State<Dishwasher> for State {
    fn call_handler(
        &mut self,
        shared: &mut Dishwasher,
        event: &Event,
        _: &mut (),
    ) -> Outcome<Self> {
        match self {
            State::DoorOpened => Dishwasher::door_opened(shared, event),
            State::Dry => Dishwasher::dry(event),
            State::Idle => Dishwasher::idle(event),
            State::Soap => Dishwasher::soap(event),
            State::Rinse => Dishwasher::rinse(event),
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

impl blocking::Superstate<Dishwasher> for Superstate {
    fn call_handler(
        &mut self,
        shared: &mut Dishwasher,
        event: &Event,
        _: &mut (),
    ) -> Outcome<State> {
        match self {
            Superstate::DoorClosed => Dishwasher::door_closed(event),
        }
    }
}

impl Dishwasher {
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::StartProgram => Transition(State::Soap),
            _ => Super,
        }
    }

    fn soap(event: &Event) -> Outcome<State> {
        match event {
            Event::TimerElapsed => Transition(State::Rinse),
            _ => Super,
        }
    }

    fn rinse(event: &Event) -> Outcome<State> {
        match event {
            Event::TimerElapsed => Transition(State::Dry),
            _ => Super,
        }
    }

    fn dry(event: &Event) -> Outcome<State> {
        match event {
            Event::TimerElapsed => Transition(State::Idle),
            _ => Super,
        }
    }

    fn door_closed(event: &Event) -> Outcome<State> {
        match event {
            // When the door is opened the program needs to be paused until
            // the door is closed again.
            Event::DoorOpened => Transition(State::DoorOpened),
            _ => Super,
        }
    }

    fn door_opened(&mut self, event: &Event) -> Outcome<State> {
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
        previous_state: Dishwasher::initial(),
    }
    .uninitialized_state_machine()
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
