#![feature(generic_associated_types)]
#![allow(unused)]

use stateful::prelude::*;
use std::io::Empty;
use std::io::Write;
use std::time::Instant;

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

#[derive(Default)]
pub struct CdPlayer;

#[derive(Debug)]
pub enum State {
    Empty,
    Open,
    Stopped,
    Playing,
    Pause,
}

// The `stateful` trait needs to be implemented on the type that will be
// the context for the state machine.
impl StateMachine for CdPlayer {
    /// The enum that represents the state.
    type State = State;

    type Superstate<'a> = ();

    /// The input type that will be submitted to the state machine.
    type Input = Event;

    type Context = Self;

    /// The initial state of the state machine.
    const INIT_STATE: State = State::Empty;
}

impl stateful::State<CdPlayer> for State {
    fn call_handler(&mut self, cd_player: &mut CdPlayer, input: &Event) -> Response<Self> {
        match self {
            State::Empty => CdPlayer::empty(input),
            State::Open => CdPlayer::open(input),
            State::Stopped => CdPlayer::stopped(input),
            State::Playing => CdPlayer::playing(input),
            State::Pause => CdPlayer::pause(input),
        }
    }
}

impl CdPlayer {
    fn empty(input: &Event) -> Response<State> {
        match input {
            Event::CdDetected => (Transition(State::Stopped)),
            Event::OpenClose => (Transition(State::Open)),
            _ => (Super),
        }
    }

    fn open(input: &Event) -> Response<State> {
        match input {
            Event::OpenClose => (Transition(State::Empty)),
            _ => (Super),
        }
    }

    fn stopped(input: &Event) -> Response<State> {
        match input {
            Event::Play => (Transition(State::Playing)),
            Event::OpenClose => (Transition(State::Open)),
            Event::Stop => (Transition(State::Stopped)),
            _ => (Super),
        }
    }

    fn playing(input: &Event) -> Response<State> {
        match input {
            Event::OpenClose => (Transition(State::Open)),
            Event::Pause2 => (Transition(State::Pause)),
            Event::Stop => (Transition(State::Stopped)),
            _ => (Super),
        }
    }

    fn pause(input: &Event) -> Response<State> {
        match input {
            Event::EndPause => (Transition(State::Playing)),
            Event::Stop => (Transition(State::Stopped)),
            _ => (Super),
        }
    }
}

fn main() {
    let mut state_machine = CdPlayer::state_machine().init();

    let loops: u32 = rand::random();

    println!("Loop count: {loops}");

    let instant = Instant::now();

    for _ in 0..loops {
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

    let total_duration = instant.elapsed();
    let loop_duration = total_duration.div_f64(loops as f64);
    let million_loop_duration = loop_duration.mul_f64(1_000_000.0);

    println!("Total duration: {total_duration:?}");
    println!("Average loop duration: {loop_duration:?}");
    println!("Duration 1M loops: {million_loop_duration:?}");

    println!("Final state: {:?}", state_machine.state());
}
