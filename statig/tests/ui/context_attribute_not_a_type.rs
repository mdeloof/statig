use statig::prelude::*;

enum Event {
    Bump,
}

#[derive(Default)]
struct Machine;

#[state_machine(initial = "State::idle()", context = "not a type")]
impl Machine {
    #[state]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Bump => Handled,
        }
    }
}

fn main() {}
