#![allow(unused)]

// The prelude module re-exports the most common used items from statig.
use statig::prelude::*;

#[derive(Default)]
pub struct Blinky {
    led: bool,
}

pub struct Event;

/// The `StateMachine` trait needs to be implemented on a custom type and
/// defines all the types associated with the state machine.
impl StateMachine for Blinky {
    /// The event type that will be submitted to the state machine.
    type Event = Event;

    /// The enum that represents the states. This type is derived by the
    /// `state_machine` macro.
    type State = State;

    /// The enum that represents the superstates. This type is derived by the
    /// `state_machine` macro. Notice that the `Superstate` associated type has
    /// a lifetime parameter. That is because a superstate is a projection of
    /// an underlying state (or superstate) and is able to borrow any fields
    /// that they define. Here we're not using any superstates so the
    /// `Superstate` enum doesn't have any variants that would require a
    /// lifetime parameter. In case they do, you'd use
    /// `type Superstate<'a> = Superstate<'a>`
    type Superstate<'a> = Superstate;

    /// The initial state of the state machine. `State::off()` is a
    /// constructor derived by `state_machine` macro.
    const INIT_STATE: State = State::off();

    /// This method is called on every transition of the state machine.
    fn on_transition(&mut self, _: &State, target: &State) {
        println!("Transitioned to `{target:?}`");
    }
}

/// The `state_machine` procedural macro generates the `State` and `Superstate`
/// enums by parsing the function signatures with a `state`, `superstate` or
/// `action` attribute. It also implements the `statig::State` and
/// `statig::Superstate` traits. We also pass an argument that will add the
/// derive macro with the Debug trait to the `State` enum.
#[state_machine(state(derive(Debug)))]
impl Blinky {
    /// The `#[state]` attibute marks this as a state handler.  By default the
    /// `event` argument will map to the event handler by the state machine.
    /// Every state must return a `Response<State>`.
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
    /// Because we're using `Blinky` as the context of the state machine
    /// we can use the `state_machine` method to turn it into a state
    /// machine.
    let state_machine = Blinky::default().state_machine();

    /// Before we submit events to the state machine we need to call the
    /// `init` method on it. This will initialized the state machine
    /// by executing all entry action into the initial state.
    let mut state_machine = state_machine.init();

    state_machine.handle(&Event);
}
