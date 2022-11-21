#![allow(unused)]

use statig::prelude::*;
use statig::StateMachine;
use statig::StateOrSuperstate;
use std::io::Write;

#[derive(Default)]
pub struct Blinky;

#[derive(Debug)]
pub struct Event;

#[state_machine(
    init = "State::on(true, 10)",
    state(derive(Debug)),
    superstate(derive(Debug)),
    on_transition = "Self::on_transition",
    on_dispatch = "Self::on_dispatch"
)]
impl Blinky {
    // Every state needs to have a `#[state]` attribute added to it.
    #[state(superstate = "playing")]
    fn on(&mut self, led: &mut bool, counter: &mut isize, event: &Event) -> Response<State> {
        Transition(State::off(false))
    }

    // Actions can access state-local storage.
    #[action]
    fn enter_off(&mut self, led: &mut bool) {
        *led = false;
    }

    #[state(superstate = "playing", entry_action = "enter_off")]
    fn off(&mut self, led: &mut bool, event: &Event) -> Response<State> {
        Transition(State::on(true, 10))
    }

    #[superstate]
    fn playing(&mut self, led: &mut bool) -> Response<State> {
        Handled
    }
}

impl Blinky {
    fn on_transition(&mut self, source: &State, target: &State) {
        println!("transition from `{:?}` to `{:?}`", source, target);
    }

    fn on_dispatch(&mut self, state: StateOrSuperstate<'_, '_, Self>, event: &Event) {
        println!("dipatching `{:?}` to `{:?}`", event, state);
    }
}

fn main() {
    let mut state_machine = Blinky::default().state_machine().init();

    for _ in 0..10 {
        state_machine.handle(&Event {});
    }
}
