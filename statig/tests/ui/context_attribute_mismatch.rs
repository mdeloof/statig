use statig::prelude::*;

struct Writer<'w> {
    log: &'w mut Vec<u32>,
}

struct Ctx<'a, 'b> {
    writer: &'a mut Writer<'b>,
}

struct Other;

enum Event {
    Bump,
}

#[derive(Default)]
struct Machine;

#[state_machine(initial = "State::idle()", context = "Ctx<'ctx, 'data>")]
impl Machine {
    #[state]
    fn idle(context: &mut Other, event: &Event) -> Outcome<State> {
        let _ = context;

        match event {
            Event::Bump => Handled,
        }
    }
}

fn main() {}
