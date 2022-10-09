#![allow(unused)]

#[cfg(test)]
mod tests {

    use stateful::prelude::*;
    use std::io::Write;

    #[derive(Default)]
    struct Blinky;

    // The `stateful` trait needs to be implemented on the type that will
    // imlement the state machine.
    impl StateMachine for Blinky {
        /// The enum that represents the state, this type is derived by the
        /// `#[state_machine]` macro.
        type State = StateEnum;

        type Superstate<'a> = Superstate<'a>;

        /// The event type that will be submitted to the state machine.
        type Event = Event;

        type Context = Self;

        /// The initial state of the state machine.
        const INIT_STATE: StateEnum = StateEnum::On {
            led: false,
            counter: 23,
        };

        fn on_transition(context: &mut Self, source: &Self::State, _target: &Self::State) {}
    }

    impl Default for StateEnum {
        fn default() -> Self {
            Blinky::INIT_STATE
        }
    }

    struct Event;

    impl Blinky {
        fn on(
            &mut self,
            led: &mut bool,
            counter: &mut isize,
            event: &Event,
        ) -> Response<StateEnum> {
            println!("On");
            Transition(StateEnum::off(false))
        }

        fn enter_off(&mut self, led: &mut bool) {
            println!("entered off");
            *led = false;
        }

        fn off(&mut self, led: &mut bool, event: &Event) -> Response<StateEnum> {
            println!("Off");
            Transition(StateEnum::on(true, 34))
        }

        fn playing(&mut self, led: &mut bool) -> Response<StateEnum> {
            Handled
        }
    }

    enum StateEnum {
        On { led: bool, counter: isize },
        Off { led: bool },
    }

    impl StateEnum {
        fn on(led: bool, counter: isize) -> Self {
            Self::On { led, counter }
        }

        fn off(led: bool) -> Self {
            Self::Off { led }
        }
    }

    impl stateful::State<Blinky> for StateEnum {
        fn call_handler(&mut self, object: &mut Blinky, event: &Event) -> Response<StateEnum>
        where
            Self: Sized,
        {
            match self {
                StateEnum::On { led, counter } => Blinky::on(object, led, counter, event),
                StateEnum::Off { led } => Blinky::off(object, led, event),
            }
        }

        fn call_entry_action(&mut self, object: &mut Blinky) {
            match self {
                StateEnum::On { led, counter } => {}
                StateEnum::Off { led } => Blinky::enter_off(object, led),
            }
        }

        fn call_exit_action(&mut self, object: &mut Blinky) {
            match self {
                StateEnum::On { led, counter } => {}
                StateEnum::Off { led } => {}
            }
        }

        fn superstate(&mut self) -> Option<Superstate> {
            match self {
                StateEnum::On { led, counter } => Some(Superstate::Playing { led }),
                StateEnum::Off { led } => Some(Superstate::Playing { led }),
            }
        }
    }

    enum Superstate<'sub> {
        Playing { led: &'sub mut bool },
    }

    impl<'sub> stateful::Superstate<Blinky> for Superstate<'sub> {
        fn call_handler(&mut self, object: &mut Blinky, event: &Event) -> Response<StateEnum> {
            match self {
                Superstate::Playing { led } => Blinky::playing(object, led),
            }
        }

        fn call_entry_action(&mut self, object: &mut Blinky) {
            match self {
                Superstate::Playing { led } => {}
            }
        }

        fn call_exit_action(&mut self, object: &mut Blinky) {
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
        let mut state_machine = Blinky::default().state_machine().init();

        for _ in 0..10 {
            state_machine.handle(&Event {});
        }
    }
}
