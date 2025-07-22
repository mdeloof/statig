#![allow(unused)]

#[cfg(test)]
mod tests {
    use statig::blocking::{self, *};

    use std::io::Write;

    #[derive(Default)]
    struct Blinky;

    // The `statig` trait needs to be implemented on the type that will
    // implement the state machine.
    impl IntoStateMachine for Blinky {
        /// The enum that represents the state, this type is derived by the
        /// `#[state_machine]` macro.
        type State = State;

        type Superstate<'sub> = Superstate<'sub>;

        /// The event type that will be submitted to the state machine.
        type Event<'evt> = Event;

        type Context<'ctx> = ();

        type Response = ();

        /// The initial state of the state machine.
        fn initial() -> Self::State {
            Superstate::playing()
        }
        
        fn default_response() -> Self::Response {
            Self::Response::default()
        }
    }

    impl Default for State {
        fn default() -> Self {
            Blinky::initial()
        }
    }

    struct Event;

    impl Blinky {
        fn on(&mut self, led: &mut bool, counter: &mut isize, event: &Event) -> Outcome<State> {
            Transition(State::off(false))
        }

        fn enter_off(&mut self, led: &mut bool) {
            println!("entered off");
            *led = false;
        }

        fn off(&mut self, led: &mut bool, event: &Event) -> Outcome<State> {
            Transition(State::on(true, 34))
        }

        fn playing(&mut self, led: &mut bool) -> Outcome<State> {
            Handled(())
        }
    }

    enum State {
        On { led: bool, counter: isize },
        Off { led: bool },
    }

    impl State {
        fn on(led: bool, counter: isize) -> Self {
            Self::On { led, counter }
        }

        fn off(led: bool) -> Self {
            Self::Off { led }
        }
    }

    impl blocking::State<Blinky> for State {
        fn call_handler(
            &mut self,
            shared_storage: &mut Blinky,
            event: &Event,
            _: &mut (),
        ) -> Outcome<State>
        where
            Self: Sized,
        {
            match self {
                State::On { led, counter } => Blinky::on(shared_storage, led, counter, event),
                State::Off { led } => Blinky::off(shared_storage, led, event),
            }
        }

        fn call_entry_action(&mut self, shared_storage: &mut Blinky, _: &mut ()) {
            match self {
                State::On { led, counter } => {}
                State::Off { led } => Blinky::enter_off(shared_storage, led),
            }
        }

        fn call_exit_action(&mut self, shared_storage: &mut Blinky, _: &mut ()) {
            match self {
                State::On { led, counter } => {}
                State::Off { led } => {}
            }
        }

        fn superstate(&mut self) -> Option<Superstate> {
            match self {
                State::On { led, counter } => Some(Superstate::Playing { led }),
                State::Off { led } => Some(Superstate::Playing { led }),
            }
        }
    }

    enum Superstate<'sub> {
        Playing { led: &'sub mut bool },
    }

    impl<'sub> Superstate<'sub> {
        fn playing() -> State {
            State::on(false, 23)
        }
    }

    impl<'sub> blocking::Superstate<Blinky> for Superstate<'sub> {
        fn call_handler(
            &mut self,
            shared_storage: &mut Blinky,
            event: &Event,
            _: &mut (),
        ) -> Outcome<State> {
            match self {
                Superstate::Playing { led } => Blinky::playing(shared_storage, led),
            }
        }

        fn call_entry_action(&mut self, shared_storage: &mut Blinky, _: &mut ()) {
            match self {
                Superstate::Playing { led } => {}
            }
        }

        fn call_exit_action(&mut self, shared_storage: &mut Blinky, _: &mut ()) {
            match self {
                Superstate::Playing { led } => {}
            }
        }

        fn superstate(&mut self) -> Option<Superstate>
        where
            Self: Sized,
        {
            match self {
                Superstate::Playing { led } => None,
            }
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
