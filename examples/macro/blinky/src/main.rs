#![allow(unused)]

use statig::prelude::*;
use statig::StateMachine;
use std::io::Write;

#[derive(Default)]
pub struct Blinky;

pub struct Event;

#[state_machine(init = "State::on(true, 10)")]
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
