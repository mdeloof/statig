#![allow(unused)]

#[cfg(test)]
mod tests {
    use statig::blocking::{self, *};

    use std::io::Write;

    #[derive(Default)]
    struct Blinky;

    struct Event;

    #[state_machine(initial = "Superstate::playing()")]
    impl Blinky {
        #[state]
        fn on(&mut self, led: &mut bool, counter: &mut isize, event: &Event) -> Response<State> {
            Transition(State::off(false))
        }

        #[action]
        fn enter_off(&mut self, led: &mut bool) {
            println!("entered off");
            *led = false;
        }

        #[state(entry_action = "enter_off")]
        fn off(&mut self, led: &mut bool, event: &Event) -> Response<State> {
            Transition(State::on(true, 34))
        }

        #[superstate(initial = "State::on(false, 23)")]
        fn playing(&mut self, led: &mut bool) -> Response<State> {
            Handled
        }
    }

    #[test]
    fn main() {
        let mut state_machine = Blinky.uninitialized_state_machine().init();

        for _ in 0..10 {
            state_machine.handle(&Event);
        }
    }
}
