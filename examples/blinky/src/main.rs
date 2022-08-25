#![feature(generic_associated_types)]
#![allow(unused)]

use stateful::prelude::*;
use stateful::StateMachine;
use std::io::Write;

#[derive(Default)]
pub struct Blinky;

pub enum State {
    On { led: bool, counter: isize },
    Off { led: bool },
}

pub enum Superstate<'a> {
    Playing { led: &'a mut bool },
}

// The `stateful` trait needs to be implemented on the type that will
// imlement the state machine.
impl StateMachine for Blinky {
    /// The enum that represents the state.
    type State = State;

    type Superstate<'a> = Superstate<'a>;

    /// The input type that will be submitted to the state machine.
    type Input = Event;

    type Context = Self;

    /// The initial state of the state machine.
    const INIT_STATE: State = State::On {
        led: false,
        counter: 10,
    };

    fn on_transition(blinky: &mut Blinky, source: &Self::State, _target: &Self::State) {}
}

impl stateful::State<Blinky> for State {
    fn call_handler(&mut self, blinky: &mut Blinky, input: &Event) -> Response<Self> {
        match self {
            State::On { led, counter } => blinky.on(led, counter, input),
            State::Off { led } => blinky.off(led, input),
        }
    }

    fn call_entry_action(&mut self, blinky: &mut Blinky) {
        match self {
            State::On { .. } => {}
            State::Off { led } => blinky.enter_off(led),
        }
    }

    fn superstate(&mut self) -> Option<Superstate<'_>> {
        match self {
            State::On { led, .. } => Some(Superstate::Playing { led }),
            State::Off { led, .. } => Some(Superstate::Playing { led }),
        }
    }
}

impl<'a> stateful::Superstate<Blinky> for Superstate<'a> {
    fn call_handler(&mut self, blinky: &mut Blinky, event: &Event) -> Response<State> {
        match self {
            Superstate::Playing { led } => blinky.playing(led),
        }
    }
}

pub struct Event;

impl Blinky {
    // Every state needs to have a `#[state]` attribute added to it.
    fn on(&mut self, led: &mut bool, counter: &mut isize, input: &Event) -> Response<State> {
        println!("On");
        Transition(State::Off { led: false })
    }

    // Actions can access state-local storage.
    fn enter_off(&mut self, led: &mut bool) {
        println!("entered off");
        *led = false;
    }

    fn off(&mut self, led: &mut bool, input: &Event) -> Response<State> {
        println!("Off");
        Transition(State::On {
            led: true,
            counter: 10,
        })
    }

    fn playing(&mut self, led: &mut bool) -> Response<State> {
        Handled
    }
}

fn main() {
    let mut state_machine = Blinky::state_machine().init();

    for _ in 0..10 {
        state_machine.handle(&Event {});
    }
}
