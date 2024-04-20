#![allow(unused)]

use statig::prelude::*;
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

#[state_machine(initial = "State::empty()", state(derive(Debug)))]
impl CdPlayer {
    #[state]
    fn empty(event: &Event) -> Response<State> {
        match event {
            Event::CdDetected => Transition(State::stopped()),
            Event::OpenClose => Transition(State::open()),
            _ => Super,
        }
    }

    #[state]
    fn open(event: &Event) -> Response<State> {
        match event {
            Event::OpenClose => Transition(State::empty()),
            _ => Super,
        }
    }

    #[state]
    fn stopped(event: &Event) -> Response<State> {
        match event {
            Event::Play => Transition(State::playing()),
            Event::OpenClose => Transition(State::open()),
            Event::Stop => Transition(State::stopped()),
            _ => Super,
        }
    }

    #[state]
    fn playing(event: &Event) -> Response<State> {
        match event {
            Event::OpenClose => Transition(State::open()),
            Event::Pause2 => Transition(State::pause()),
            Event::Stop => Transition(State::stopped()),
            _ => Super,
        }
    }

    #[state]
    fn pause(event: &Event) -> Response<State> {
        match event {
            Event::EndPause => Transition(State::playing()),
            Event::Stop => Transition(State::stopped()),
            _ => Super,
        }
    }
}

fn main() {
    let mut state_machine = CdPlayer.uninitialized_state_machine().init();

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
