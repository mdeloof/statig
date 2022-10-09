#![allow(unused)]

use stateful::prelude::*;

#[derive(Default)]
pub struct Blinky {
    led: bool,
}

pub struct Event;

// The `stateful` trait needs to be implemented on the type that will be
// the context for the state machine.
impl StateMachine for Blinky {
    /// The enum that represents the states.
    type State = State;

    /// The enum that represents the superstates.
    type Superstate<'a> = Superstate;

    /// The event type that will be submitted to the state machine.
    type Event = Event;

    /// As a context we use the [Blinky] struct itself.
    type Context = Self;

    /// The initial state of the state machine.
    const INIT_STATE: State = State::off();

    /// This method is called on every transition of the state machine.
    fn on_transition(_: &mut Blinky, _: &State, target: &State) {
        println!("Transitioned to `{target:?}`");
    }
}

/// The `state_machine` proc macro generates the `State` and `Superstate` enums
/// by parsing the function signatures with a `state`, `superstate` or `action`
/// attribute.
#[state_machine(state(derive(Debug)))]
impl Blinky {
    #[state]
    fn on(&mut self, event: &Event) -> Response<State> {
        self.led = false;
        // Transition to the `off` state.
        Transition(State::off())
    }

    #[state]
    fn off(&mut self, event: &Event) -> Response<State> {
        self.led = true;
        // Transition to the `on` state.
        Transition(State::on())
    }
}

fn main() {
    let mut state_machine = Blinky::default().state_machine().init();

    state_machine.handle(&Event);
}
