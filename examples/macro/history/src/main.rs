use statig::blocking::*;

pub enum Event {
    StartProgram,
    DoorOpened,
    DoorClosed,
    TimerElapsed,
}

#[derive(Debug)]
pub struct Dishwasher {
    previous_state: State,
}

#[state_machine(
    initial = "State::idle()",
    after_transition = "Self::after_transition",
    state(derive(Debug, Clone))
)]
impl Dishwasher {
    // On every transition we update the previous state, so we can transition back to it.
    fn after_transition(&mut self, source: &State, _target: &State) {
        // println!("transitioned from `{:?}` to `{:?}`", source, _target);
        self.previous_state = source.clone();
    }

    #[superstate]
    fn door_closed(event: &Event) -> Response<State> {
        match event {
            // When the door is opened the program needs to be paused until
            // the door is closed again.
            Event::DoorOpened => Transition(State::door_opened()),
            _ => Super,
        }
    }

    #[state(superstate = "door_closed")]
    fn idle(event: &Event) -> Response<State> {
        match event {
            Event::StartProgram => Transition(State::soap()),
            _ => Super,
        }
    }

    #[state(superstate = "door_closed")]
    fn soap(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::rinse()),
            _ => Super,
        }
    }

    #[state(superstate = "door_closed")]
    fn rinse(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::dry()),
            _ => Super,
        }
    }

    #[state(superstate = "door_closed")]
    fn dry(event: &Event) -> Response<State> {
        match event {
            Event::TimerElapsed => Transition(State::idle()),
            _ => Super,
        }
    }

    #[state]
    fn door_opened(&self, event: &Event) -> Response<State> {
        match event {
            // When the door is closed again, the program is resumed from
            // the previous state.
            Event::DoorClosed => Transition(self.previous_state.clone()),
            _ => Super,
        }
    }
}

fn main() {
    let mut state_machine = Dishwasher {
        previous_state: Dishwasher::INITIAL(),
    }
    .uninitialized_state_machine()
    .init();

    println!("State: {:?}", state_machine.state()); // State: Idle

    state_machine.handle(&Event::StartProgram);

    println!("State: {:?}", state_machine.state()); // State: Soap

    state_machine.handle(&Event::TimerElapsed);

    println!("State: {:?}", state_machine.state()); // State: Rinse

    state_machine.handle(&Event::TimerElapsed);

    println!("State: {:?}", state_machine.state()); // State: Dry

    state_machine.handle(&Event::DoorOpened);

    println!("State: {:?}", state_machine.state()); // State: DoorOpened

    state_machine.handle(&Event::TimerElapsed);

    println!("State: {:?}", state_machine.state()); // State: DoorOpened

    state_machine.handle(&Event::DoorClosed);

    println!("State: {:?}", state_machine.state()); // State: Dry

    state_machine.handle(&Event::TimerElapsed);

    println!("State: {:?}", state_machine.state()); // State: Idle
}
