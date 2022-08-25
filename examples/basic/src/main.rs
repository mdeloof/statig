#![feature(generic_associated_types)]
#![allow(unused)]

use stateful::prelude::*;

#[derive(Default)]
pub struct Blinky {
    led: bool,
}

#[derive(Debug)]
pub enum State {
    On,
    Off,
}

pub struct Event;

// The `stateful` trait needs to be implemented on the type that will be
// the context for the state machine.
impl StateMachine for Blinky {
    /// The enum that represents the state.
    type State = State;

    /// We are not using any superstates for this state machine, so we set it to `()`.
    type Superstate<'a> = ();

    /// The input type that will be submitted to the state machine.
    type Input = Event;

    /// As a context we use the [Blinky] struct itself.
    type Context = Self;

    /// The initial state of the state machine.
    const INIT_STATE: State = State::Off;

    /// This method is called on every transition of the state machine.
    fn on_transition(_: &mut Blinky, _: &State, target: &State) {
        println!("Transitioned to `{target:?}`");
    }
}

impl stateful::State<Blinky> for State {
    fn call_handler(&mut self, blinky: &mut Blinky, event: &Event) -> Response<Self> {
        match self {
            State::On => blinky.on(event),
            State::Off => blinky.off(event),
        }
    }
}

impl Blinky {
    fn on(&mut self, input: &Event) -> Response<State> {
        self.led = false;
        // Transition to the `off` state.
        Transition(State::Off)
    }

    fn off(&mut self, input: &Event) -> Response<State> {
        self.led = true;
        // Transition to the `on` state.
        Transition(State::On)
    }
}

fn main() {
    let mut state_machine = Blinky::state_machine().init();

    state_machine.handle(&Event);
}
