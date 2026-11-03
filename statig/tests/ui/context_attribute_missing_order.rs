use statig::prelude::*;

struct Writer<'w> {
    log: &'w mut Vec<u32>,
}

struct Ctx<'a, 'b> {
    writer: &'a mut Writer<'b>,
}

enum Event {
    Bump,
}

#[derive(Default)]
struct Machine;

// The superstate is declared first, but the error must point at the first state handler that
// takes the context in source order (`busy`), not at the superstate or at `done`.
#[state_machine(initial = "State::idle()")]
impl Machine {
    #[superstate]
    fn on(context: &mut Ctx<'_, '_>, event: &Event) -> Outcome<State> {
        match event {
            Event::Bump => {
                context.writer.log.push(1);

                Handled
            }
        }
    }

    #[state(superstate = "on")]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Bump => Transition(State::busy()),
        }
    }

    #[state(superstate = "on")]
    fn busy(context: &mut Ctx<'_, '_>, event: &Event) -> Outcome<State> {
        match event {
            Event::Bump => {
                context.writer.log.push(2);

                Transition(State::done())
            }
        }
    }

    #[state(superstate = "on")]
    fn done(context: &mut Ctx<'_, '_>, event: &Event) -> Outcome<State> {
        match event {
            Event::Bump => {
                context.writer.log.push(3);

                Handled
            }
        }
    }
}

fn main() {}
