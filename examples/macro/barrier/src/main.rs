use statig::prelude::*;

#[derive(Default)]
struct Foo;

enum Event {
    A,
    B,
    C,
}

#[state_machine(initial = "State::initializing()", state(derive(PartialEq, Eq, Debug)))]
impl Foo {
    #[state(superstate = "waiting_for_initialization", default("a", "b", "c"))]
    fn initializing(a: &mut bool, b: &mut bool, c: &mut bool, event: &Event) -> Response<State> {
        match event {
            Event::A => {
                *a = true;
                Super
            }
            Event::B => {
                *b = true;
                Super
            }
            Event::C => {
                *c = true;
                Super
            }
        }
    }

    #[superstate]
    fn waiting_for_initialization(a: &bool, b: &bool, c: &bool) -> Response<State> {
        match (a, b, c) {
            (true, true, true) => Transition(State::initialized()),
            _ => Handled,
        }
    }

    #[state]
    fn initialized() -> Response<State> {
        Handled
    }
}

fn main() {
    let mut state_machine = Foo::default().uninitialized_state_machine().init();

    state_machine.handle(&Event::A);
    state_machine.handle(&Event::B);
    state_machine.handle(&Event::C);

    assert_eq!(state_machine.state(), &State::initialized());
}
