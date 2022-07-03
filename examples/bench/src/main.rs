#![feature(generic_associated_types)]
#![allow(unused)]

use stateful::state_machine;
use stateful::Response::*;
use stateful::Result;
use stateful::ResultExt;
use stateful::{StateMachine, Stateful};
use std::io::Write;
use std::time::Instant;

pub struct Blinky;

pub enum Event {
    StartPlayback,
    ResumePlayback,
    CloseDrawer,
    OpenDrawer,
    StopAndOpen,
    StoppedAgain,
    StoreCdInfo,
    PausePlayback,
    StopPlayback,
    Play,
    EndPause,
    Stop,
    Pause2,
    OpenClose,
    CdDetected,
}

// The `stateful` trait needs to be implemented on the type that will be
// the context for the state machine.
impl Stateful for Blinky {
    /// The enum that represents the state. This type is derived by the
    /// `#[state_machine]` macro.
    type State = State;

    /// The input type that will be submitted to the state machine.
    type Input = Event;

    /// The initial state of the state machine.
    const INIT_STATE: State = State::empty();
}

#[state_machine]
#[state(derive(Debug))]
impl Blinky {
    #[state]
    fn empty(input: &Event) -> Result<State> {
        match input {
            Event::CdDetected => Ok(Transition(State::stopped())),
            Event::OpenClose => Ok(Transition(State::open())),
            _ => Ok(Super),
        }
    }

    #[state]
    fn open(input: &Event) -> Result<State> {
        match input {
            Event::OpenClose => Ok(Transition(State::empty())),
            _ => Ok(Super),
        }
    }

    #[state]
    fn stopped(input: &Event) -> Result<State> {
        match input {
            Event::Play => Ok(Transition(State::playing())),
            Event::OpenClose => Ok(Transition(State::open())),
            Event::Stop => Ok(Transition(State::stopped())),
            _ => Ok(Super),
        }
    }

    #[state]
    fn playing(input: &Event) -> Result<State> {
        match input {
            Event::OpenClose => Ok(Transition(State::open())),
            Event::Pause2 => Ok(Transition(State::pause())),
            Event::Stop => Ok(Transition(State::stopped())),
            _ => Ok(Super),
        }
    }

    #[state]
    fn pause(input: &Event) -> Result<State> {
        match input {
            Event::EndPause => Ok(Transition(State::playing())),
            Event::Stop => Ok(Transition(State::stopped())),
            _ => Ok(Super),
        }
    }
}

fn main() {
    let mut state_machine = StateMachine::new(Blinky);

    let instant = Instant::now();

    for _ in 0..1_000_000 {
        state_machine.handle(&Event::OpenClose);
        state_machine.handle(&Event::OpenClose);
        state_machine.handle(&Event::CdDetected);
        state_machine.handle(&Event::Play);
        state_machine.handle(&Event::Pause2);

        state_machine.handle(&Event::EndPause);
        state_machine.handle(&Event::Pause2);
        state_machine.handle(&Event::Stop);

        state_machine.handle(&Event::Stop);
        state_machine.handle(&Event::OpenClose);
        state_machine.handle(&Event::OpenClose);
    }

    println!("Duration: {:?}", instant.elapsed());

    println!("Final state: {:?}", state_machine.state());
}
