#![allow(unused)]

use stateful::prelude::*;
use stateful::StateMachine;
use std::io::Write;

#[derive(Default)]
pub struct Blinky;

// The `stateful` trait needs to be implemented on the type that will
// imlement the state machine.
impl StateMachine for Blinky {
    /// The enum that represents the state.
    type State = State;

    type Superstate<'a> = Superstate<'a>;

    /// The event type that will be submitted to the state machine.
    type Event = Event;

    type Context = Self;

    /// The initial state of the state machine.
    const INIT_STATE: State = State::on(false, 10);

    fn on_transition(blinky: &mut Blinky, source: &Self::State, _target: &Self::State) {}
}

pub struct Event;

#[state_machine]
impl Blinky {
    // Every state needs to have a `#[state]` attribute added to it.
    #[state(superstate = "playing")]
    fn on(&mut self, led: &mut bool, counter: &mut isize, event: &Event) -> Response<State> {
        println!("On");
        Transition(State::off(false))
    }

    // Actions can access state-local storage.
    #[action]
    fn enter_off(&mut self, led: &mut bool) {
        println!("entered off");
        *led = false;
    }

    #[state(superstate = "playing", entry_action = "enter_off")]
    fn off(&mut self, led: &mut bool, event: &Event) -> Response<State> {
        println!("Off");
        Transition(State::on(true, 10))
    }

    #[superstate]
    fn playing(&mut self, led: &mut bool) -> Response<State> {
        Handled
    }
}

fn main() {
    let mut state_machine = Blinky::default().state_machine().init();

    for _ in 0..10 {
        state_machine.handle(&Event {});
    }
}
