#![allow(unused)]

use statig::blocking::{self, *};
use std::io::Write;

#[derive(Default)]
pub struct Blinky;

// The event that will be handled by the state machine.
pub enum Event {
    TimerElapsed,
    ButtonPressed,
}

// The enum representing all states of the state machine. These are
// the states you can actually transition to.
pub enum State {
    LedOn,
    LedOff,
    NotBlinking,
}

// The enum representing the superstates of the system. You can not transition
// to a superstate, but instead they define shared behavior of underlying states or
// superstates.
pub enum Superstate {
    Blinking,
}

// The `statig` trait needs to be implemented on the type that will
// imlement the state machine.
impl IntoStateMachine for Blinky {
    /// The enum that represents the state.
    type State = State;

    type Superstate<'sub> = Superstate;

    /// The event type that will be submitted to the state machine.
    type Event<'evt> = Event;

    type Context<'ctx> = ();

    /// The initial state of the state machine.
    fn initial() -> State {
        State::LedOn
    }
}

// Implement the `statig::State` trait for the state enum.
impl blocking::State<Blinky> for State {
    fn call_handler(&mut self, blinky: &mut Blinky, event: &Event, _: &mut ()) -> Response<Self> {
        match self {
            State::LedOn => Blinky::led_on(event),
            State::LedOff => Blinky::led_off(event),
            State::NotBlinking => Blinky::not_blinking(event),
        }
    }

    fn superstate(&mut self) -> Option<Superstate> {
        match self {
            State::LedOn => Some(Superstate::Blinking),
            State::LedOff => Some(Superstate::Blinking),
            State::NotBlinking => None,
        }
    }
}

// Implement the `statig::Superstate` trait for the superstate enum.
impl blocking::Superstate<Blinky> for Superstate {
    fn call_handler(&mut self, blinky: &mut Blinky, event: &Event, _: &mut ()) -> Response<State> {
        match self {
            Superstate::Blinking => Blinky::blinking(event),
        }
    }
}

impl Blinky {
    fn led_on(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::LedOff),
            _ => Super,
        }
    }

    fn led_off(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::LedOn),
            _ => Super,
        }
    }

    fn blinking(event: &Event) -> Response<State> {
        match event {
            Event::ButtonPressed => Transition(State::NotBlinking),
            _ => Super,
        }
    }

    fn not_blinking(event: &Event) -> Response<State> {
        match event {
            Event::ButtonPressed => Transition(State::LedOn),
            _ => Super,
        }
    }
}

fn main() {
    let mut state_machine = Blinky.state_machine();

    state_machine.handle(&Event::TimerElapsed);
    state_machine.handle(&Event::ButtonPressed);
    state_machine.handle(&Event::TimerElapsed);
    state_machine.handle(&Event::ButtonPressed);
}
