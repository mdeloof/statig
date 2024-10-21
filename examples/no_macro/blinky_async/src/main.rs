#![allow(unused)]

use statig::awaitable::{self, *};
use std::{future::Future, io::Write, pin::Pin};

#[derive(Default)]
pub struct Blinky;

// The event that will be handled by the state machine.
pub enum Event {
    TimerElapsed,
    ButtonPressed,
}

// The enum representing all states of the state machine. These are
// the states you can actually transition to.
#[derive(Debug)]
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
    const INITIAL: State = State::LedOn;

}

impl IntoStateMachineExt for Blinky {
    
    async fn on_transition(&mut self, from_state: &State, to_state: &State) {
        println!("Transitioned to {:?}", to_state);
    }

}

// Implement the `statig::State` trait for the state enum.
impl awaitable::State<Blinky> for State {
    fn call_handler<'fut>(&'fut mut self, blinky: &'fut mut Blinky, event: &'fut Event, _: &'fut mut ()) -> Pin<Box<(dyn Future<Output = statig::Response<State>> + Send + 'fut)>>{
        match self {
            State::LedOn => Box::pin(Blinky::led_on(event)),
            State::LedOff => Box::pin(Blinky::led_off(event)),
            State::NotBlinking => Box::pin(Blinky::not_blinking(event)),
        }
    }

    fn superstate<'fut>(&mut self) -> Option<Superstate> {
        match self {
            State::LedOn => Some(Superstate::Blinking),
            State::LedOff => Some(Superstate::Blinking),
            State::NotBlinking => None,
        }
    }
}

// Implement the `statig::Superstate` trait for the superstate enum.
impl awaitable::Superstate<Blinky> for Superstate {
    fn call_handler<'fut>(&'fut mut self, blinky: &'fut mut Blinky, event: &'fut Event, _: &'fut mut ()) -> Pin<Box<(dyn Future<Output= Response<State>> + Send + 'fut)>> {
        Box::pin(match self {
            Superstate::Blinking => Blinky::blinking(event),
        })
    }
}

impl Blinky {
    async fn led_on(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::LedOff),
            _ => Super,
        }
    }

    async fn led_off(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::LedOn),
            _ => Super,
        }
    }

    async fn blinking(event: &Event) -> Response<State> {
        match event {
            Event::ButtonPressed => Transition(State::NotBlinking),
            _ => Super,
        }
    }

    async fn not_blinking(event: &Event) -> Response<State> {
        match event {
            Event::ButtonPressed => Transition(State::LedOn),
            _ => Super,
        }
    }

}

#[tokio::main]
async fn main() {
    let mut state_machine = Blinky::default().state_machine();

    state_machine.handle(&Event::TimerElapsed).await;
    state_machine.handle(&Event::ButtonPressed).await;
    state_machine.handle(&Event::TimerElapsed).await;
    state_machine.handle(&Event::ButtonPressed).await;
}
