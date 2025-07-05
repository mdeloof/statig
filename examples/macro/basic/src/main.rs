#![allow(unused)]

// The prelude module re-exports the most common used items from statig.
use statig::prelude::*;

#[derive(Default)]
pub struct Blinky {
    led: bool,
}

pub struct Event;

/// The `state_machine` procedural macro generates the `State` and `Superstate`
/// enums by parsing the function signatures with a `state`, `superstate` or
/// `action` attribute. It also implements the `statig::State` and
/// `statig::Superstate` traits. We also pass an argument that will add the
/// derive macro with the Debug trait to the `State` enum.
#[state_machine(initial = State::off(), state(derive(Debug)))]
impl Blinky {
    /// The `#[state]` attribute marks this as a state handler.  By default the
    /// `event` argument will map to the event handler by the state machine.
    /// Every state must return a `Outcome<State>`.
    #[state]
    fn on(&mut self, event: &Event) -> Outcome<State> {
        self.led = false;
        // Transition to the `off` state.
        Transition(State::off())
    }

    #[state]
    fn off(&mut self, event: &Event) -> Outcome<State> {
        self.led = true;
        // Transition to the `on` state.
        Transition(State::on())
    }
}

fn main() {
    /// We use the `state_machine` method to create a state machine.
    let state_machine = Blinky::default().uninitialized_state_machine();

    /// Before we submit events to the state machine we need to call the
    /// `init` method on it. This will initialized the state machine
    /// by executing all entry action into the initial state.
    let mut state_machine = state_machine.init();

    state_machine.handle(&Event);
}
