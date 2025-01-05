#![allow(unused)]

use statig::blocking::{self, *};

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

/// The `StateMachine` trait needs to be implemented on the type that will be
/// the shared storage for the state machine.
impl IntoStateMachine for Blinky {
    /// The enum that represents the state.
    type State = State;

    /// We are not using any superstates for this state machine, so we set it to `()`.
    type Superstate<'sub> = ();

    /// The event type that will be submitted to the state machine.
    type Event<'evt> = Event;

    type Context<'ctx> = ();

    /// The initial state of the state machine.
    const INITIAL: State = State::Off;

    /// This method is called on every transition of the state machine.
    const after_transition: fn(&mut Self, &Self::State, &Self::State) = |_, source, target| {
        println!("transitioned from {source:?} to {target:?}");
    };
}

impl blocking::State<Blinky> for State {
    fn call_handler(&mut self, blinky: &mut Blinky, event: &Event, _: &mut ()) -> Response<Self> {
        match self {
            State::On => blinky.on(event),
            State::Off => blinky.off(event),
        }
    }
}

impl Blinky {
    fn on(&mut self, event: &Event) -> Response<State> {
        self.led = false;
        // Transition to the `off` state.
        Transition(State::Off)
    }

    fn off(&mut self, event: &Event) -> Response<State> {
        self.led = true;
        // Transition to the `on` state.
        Transition(State::On)
    }
}

fn main() {
    let mut state_machine = Blinky::default().uninitialized_state_machine().init();

    state_machine.handle(&Event);
}
