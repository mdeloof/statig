#![allow(unused)]

use futures::executor;
use statig::awaitable::IntoStateMachine; // Use this instead of `statig::blocking::IntoStateMachine`.
use statig::prelude::*;
use std::fmt::Debug;
use std::future::Future;
use std::io::Write;
use std::thread::spawn;

#[derive(Debug, Default)]
pub struct Blinky;

// The event that will be handled by the state machine.
#[derive(Debug)]
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
#[derive(Debug)]
pub enum Superstate {
    Blinking,
}

// The `statig` trait needs to be implemented on the type that will
// implement the state machine.
impl IntoStateMachine for Blinky {
    /// The event type that will be submitted to the state machine.
    type Event<'evt> = Event;

    type Context<'ctx> = ();

    /// The enum that represents the state.
    type State = State;

    type Superstate<'sub> = Superstate;

    /// The initial state of the state machine.
    fn initial() -> State {
        State::LedOn
    }

    async fn before_dispatch(
        &mut self,
        state_or_superstate: StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
        event: &Self::Event<'_>,
        context: &mut Self::Context<'_>,
    ) {
        self.before_dispatch(state_or_superstate, event, context)
            .await
    }

    async fn after_dispatch(
        &mut self,
        state_or_superstate: StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
        event: &Self::Event<'_>,
        context: &mut Self::Context<'_>,
    ) {
        self.after_dispatch(state_or_superstate, event, context)
            .await
    }

    async fn before_transition(
        &mut self,
        source: &Self::State,
        target: &Self::State,
        context: &mut Self::Context<'_>,
    ) {
        self.before_transition(source, target, context).await
    }

    async fn after_transition(
        &mut self,
        source: &Self::State,
        target: &Self::State,
        context: &mut Self::Context<'_>,
    ) {
        self.after_transition(source, target, context).await
    }
}

// Implement the `statig::State` trait for the state enum.
impl awaitable::State<Blinky> for State {
    async fn call_handler(
        &mut self,
        shared_storage: &mut Blinky,
        event: &<Blinky as IntoStateMachine>::Event<'_>,
        context: &mut <Blinky as IntoStateMachine>::Context<'_>,
    ) -> Outcome<Self> {
        match self {
            State::LedOn => Blinky::led_on(event).await,
            State::LedOff => Blinky::led_off(event),
            State::NotBlinking => Blinky::not_blinking(event).await,
        }
    }

    async fn call_entry_action(
        &mut self,
        shared_storage: &mut Blinky,
        context: &mut <Blinky as IntoStateMachine>::Context<'_>,
    ) {
        match self {
            State::LedOn => Blinky::cool().await,
            _ => {}
        }
    }

    fn superstate(&mut self) -> Option<<Blinky as IntoStateMachine>::Superstate<'_>> {
        match self {
            State::LedOn => Some(Superstate::Blinking),
            State::LedOff => Some(Superstate::Blinking),
            State::NotBlinking => None,
        }
    }
}

// Implement the `statig::Superstate` trait for the superstate enum.
impl awaitable::Superstate<Blinky> for Superstate {
    async fn call_handler(
        &mut self,
        shared_storage: &mut Blinky,
        event: &<Blinky as IntoStateMachine>::Event<'_>,
        context: &mut <Blinky as IntoStateMachine>::Context<'_>,
    ) -> Outcome<<Blinky as IntoStateMachine>::State> {
        match self {
            Superstate::Blinking => Blinky::blinking(event).await,
        }
    }
}

impl Blinky {
    async fn cool() {}

    async fn led_on(event: &Event) -> Outcome<State> {
        match event {
            Event::TimerElapsed => Transition(State::LedOff),
            _ => Super,
        }
    }

    /// Note you can mix sync and async handlers/actions.
    fn led_off(event: &Event) -> Outcome<State> {
        match event {
            Event::TimerElapsed => Transition(State::LedOn),
            _ => Super,
        }
    }

    async fn blinking(event: &Event) -> Outcome<State> {
        match event {
            Event::ButtonPressed => Transition(State::NotBlinking),
            _ => Super,
        }
    }

    async fn not_blinking(event: &Event) -> Outcome<State> {
        match event {
            Event::ButtonPressed => Transition(State::LedOn),
            // Although this state has no superstate, we can still defer the event which
            // will cause the event to be handled by an implicit `top` superstate.
            _ => Super,
        }
    }
}

impl Blinky {
    // The `after_transition` callback that will be called after every transition.
    async fn after_transition(&mut self, source: &State, target: &State, _context: &mut ()) {
        println!("after transitioned from `{source:?}` to `{target:?}`");
    }

    async fn before_transition(&mut self, source: &State, target: &State, _context: &mut ()) {
        println!("before transitioned from `{source:?}` to `{target:?}`");
    }

    async fn before_dispatch(
        &mut self,
        state: StateOrSuperstate<'_, State, Superstate>,
        event: &Event,
        _context: &mut (),
    ) {
        println!("before dispatching `{event:?}` to `{state:?}`");
    }

    async fn after_dispatch(
        &mut self,
        state: StateOrSuperstate<'_, State, Superstate>,
        event: &Event,
        _context: &mut (),
    ) {
        println!("after dispatched `{event:?}` to `{state:?}`");
    }
}

#[tokio::main]
async fn main() {
    use tokio::task;

    let future = async move {
        let mut state_machine = Blinky.state_machine();

        state_machine.handle(&Event::TimerElapsed).await;
        state_machine.handle(&Event::ButtonPressed).await;
        state_machine.handle(&Event::TimerElapsed).await;
        state_machine.handle(&Event::ButtonPressed).await;
    };

    let local = task::LocalSet::new();

    let handle = local.run_until(future);

    handle.await;
}
