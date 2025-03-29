#![allow(unused)]

use futures::executor;
use statig::prelude::*;
use std::fmt::Debug;
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

/// The `state_machine` procedural macro generates the `State` and `Superstate`
/// enums by parsing the function signatures with a `state`, `superstate` or
/// `action` attribute. It also implements the `statig::State` and
/// `statig::Superstate` traits.
#[state_machine(
    // This sets the initial state to `led_on`.
    initial = "State::led_on(2)",
    // Derive the Debug trait on the `State` enum.
    state(derive(Debug)),
    // Derive the Debug trait on the `Superstate` enum.
    superstate(derive(Debug)),
    // Set the `after_transition` callback.
    after_transition = "Self::after_transition",
    // Set the `before_dispatch` callback.
    before_dispatch = "Self::before_dispatch",
    // Set the `after_dispatch` callback.
    after_dispatch = "Self::after_dispatch",
    // Set the `before_transition` callback.
    before_transition = "Self::before_transition"
)]
impl Blinky {
    #[action]
    fn cool() {}
    /// The `#[state]` attibute marks this as a state handler.  By default the
    /// `event` argument will map to the event handler by the state machine.
    /// Every state must return a `Response<State>`.
    #[state(superstate = "blinking", entry_action = "cool")]
    async fn led_on(value: &i32, event: &Event) -> Response<State> {
        match event {
            // When we receive a `TimerElapsed` event we transition to the `led_off` state.
            Event::TimerElapsed => Transition(State::led_off(2)),
            // Other events are deferred to the superstate, in this case `blinking`.
            _ => Super,
        }
    }

    /// Note you can mix sync and async handlers/actions.
    #[state(superstate = "blinking")]
    fn led_off(value: &i32, event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::led_on(2)),
            _ => Super,
        }
    }

    /// The `#[superstate]` attribute marks this as a superstate handler.
    #[superstate]
    async fn blinking(value: &i32, event: &Event) -> Response<State> {
        match event {
            Event::ButtonPressed => Transition(State::not_blinking()),
            _ => Super,
        }
    }

    #[state]
    async fn not_blinking(event: &Event) -> Response<State> {
        match event {
            Event::ButtonPressed => Transition(State::led_on(2)),
            // Altough this state has no superstate, we can still defer the event which
            // will cause the event to be handled by an implicit `top` superstate.
            _ => Super,
        }
    }
}

impl Blinky {
    // The `after_transition` callback that will be called after every transition.
    async fn after_transition(&mut self, source: &State, target: &State) {
        println!("after transitioned from `{source:?}` to `{target:?}`");
    }

    async fn before_transition(&mut self, source: &State, target: &State) {
        println!("before transitioned from `{source:?}` to `{target:?}`");
    }

    async fn before_dispatch(
        &mut self,
        state: StateOrSuperstate<'_, State, Superstate<'_>>,
        event: &Event,
    ) {
        println!("before dispatching `{event:?}` to `{state:?}`");
    }

    async fn after_dispatch(
        &mut self,
        state: StateOrSuperstate<'_, State, Superstate<'_>>,
        event: &Event,
    ) {
        println!("after dispatched `{event:?}` to `{state:?}`");
    }

    // // The `on_transition` callback that will be called after every transition.
    // async fn on_transition(&mut self, source: &State, target: &State) {
    //     println!("transitioned from `{source:?}` to `{target:?}`");
    // }

    // async fn on_dispatch(
    //     &mut self,
    //     state: StateOrSuperstate<'_, State, Superstate<'_>>,
    //     event: &Event,
    // ) {
    //     println!("dispatching `{event:?}` to `{state:?}`");

    // }
}

#[tokio::main]
async fn main() {
    use tokio::task;

    let future = async move {
        let mut state_machine = Blinky.state_machine(); //.uninitialized_state_machine().init().await;

        state_machine.handle(&Event::TimerElapsed).await;
        state_machine.handle(&Event::ButtonPressed).await;
        state_machine.handle(&Event::TimerElapsed).await;
        state_machine.handle(&Event::ButtonPressed).await;
    };

    let local = task::LocalSet::new();

    let handle = local.run_until(future);

    handle.await;
}
