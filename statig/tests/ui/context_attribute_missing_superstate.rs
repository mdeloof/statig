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

// No state handler takes the context, so the error must point at the superstate handler.
#[state_machine(initial = "State::idle()")]
impl Machine {
    #[state(superstate = "on")]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Bump => Super,
        }
    }

    #[superstate]
    fn on(context: &mut Ctx<'_, '_>, event: &Event) -> Outcome<State> {
        match event {
            Event::Bump => {
                context.writer.log.push(1);

                Handled
            }
        }
    }
}

fn main() {}
