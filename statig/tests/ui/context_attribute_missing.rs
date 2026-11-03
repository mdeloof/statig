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

#[state_machine(initial = "State::idle()")]
impl Machine {
    #[state]
    fn idle(context: &mut Ctx<'_, '_>, event: &Event) -> Outcome<State> {
        match event {
            Event::Bump => {
                context.writer.log.push(10);

                Handled
            }
        }
    }
}

fn main() {}
