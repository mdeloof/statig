#[test]
#[cfg(feature = "serde")]
fn serialize_deserialize() {
    #![allow(unused)]

    use std::fmt::Debug;
    use std::io::Write;

    use serde::{Deserialize, Serialize};
    use statig::blocking::StateOrSuperstate;
    use statig::prelude::*;

    #[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
    pub struct Blinky {
        led: bool,
    }

    #[derive(Debug)]
    pub enum Event {
        TimerElapsed,
        ButtonPressed,
    }

    #[state_machine(
        initial = State::led_on(),
        state(derive(Debug, Serialize, Deserialize, Clone, PartialEq)),
        superstate(derive(Debug))
    )]
    impl Blinky {
        #[state(superstate = blinking)]
        fn led_on(event: &Event) -> Outcome<State> {
            match event {
                Event::TimerElapsed => Transition(State::led_off()),
                _ => Super,
            }
        }

        #[state(superstate = blinking)]
        fn led_off(event: &Event) -> Outcome<State> {
            match event {
                Event::TimerElapsed => Transition(State::led_on()),
                _ => Super,
            }
        }

        #[superstate]
        fn blinking(event: &Event) -> Outcome<State> {
            match event {
                Event::ButtonPressed => Transition(State::not_blinking()),
                _ => Super,
            }
        }

        #[state]
        fn not_blinking(event: &Event) -> Outcome<State> {
            match event {
                Event::ButtonPressed => Transition(State::led_on()),
                _ => Super,
            }
        }
    }

    let state_machine = Blinky { led: true }.uninitialized_state_machine();
    let state_machine_init = state_machine.clone().init();
    let mut state_machine_not_blinking = state_machine_init.clone();
    state_machine_not_blinking.handle(&Event::ButtonPressed);

    let ser = serde_json::to_string(&state_machine).unwrap();
    let de: statig::blocking::UninitializedStateMachine<Blinky> =
        serde_json::from_str(&ser).unwrap();

    assert_eq!(de, state_machine);

    let ser = serde_json::to_string(&state_machine_init).unwrap();
    let de: statig::blocking::UninitializedStateMachine<Blinky> =
        serde_json::from_str(&ser).unwrap();

    assert_eq!(de, state_machine);

    let ser = bincode::serialize(&state_machine).unwrap();
    let de: statig::blocking::UninitializedStateMachine<Blinky> =
        bincode::deserialize(&ser).unwrap();

    assert_eq!(de, state_machine);

    let ser = bincode::serialize(&state_machine_init).unwrap();
    let de: statig::blocking::UninitializedStateMachine<Blinky> =
        bincode::deserialize(&ser).unwrap();

    assert_eq!(de, state_machine);

    let ser = bincode::serialize(&state_machine_not_blinking).unwrap();
    let mut de: statig::blocking::UninitializedStateMachine<Blinky> =
        bincode::deserialize(&ser).unwrap();
    let de = de.init();

    assert_eq!(de, state_machine_not_blinking);
}
