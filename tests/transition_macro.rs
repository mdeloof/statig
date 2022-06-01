#![feature(generic_associated_types)]
#![allow(unused)]

mod tests {

    use stateful::state_machine;
    use stateful::Response::*;
    use stateful::StateMachine;
    use stateful::Stateful;
    use std::fmt;

    type Result = stateful::Result<State>;

    #[derive(Clone)]
    pub enum Event {
        A,
        B,
        C,
        D,
    }

    #[derive(Copy, Clone, Debug)]
    pub enum Action {
        Entry,
        Exit,
    }

    #[derive(Debug)]
    enum StateWrapper {
        Leaf(State),
        Super(Superstate),
    }

    struct Foo {
        pub path: Vec<(StateWrapper, Action)>,
    }

    impl Stateful for Foo {
        type State = State;

        type Input = Event;

        const INIT_STATE: State = State::s11();
    }

    #[state_machine]
    #[state(derive(Clone, Copy, Debug, PartialEq))]
    #[superstate(derive(Clone, Copy, Debug, PartialEq))]
    impl Foo {
        /// s11
        #[state(
            superstate = "s1",
            entry_action = "enter_s11",
            exit_action = "exit_s11"
        )]
        pub fn s11(&mut self, input: &Event) -> Result {
            match input {
                Event::A => Ok(Transition(State::s11())),
                Event::B => Ok(Transition(State::s12())),
                _ => Ok(Super),
            }
        }

        #[action]
        fn enter_s11(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::s11()), Action::Entry));
        }

        #[action]
        fn exit_s11(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::s11()), Action::Exit));
        }

        /// s12
        #[state(
            superstate = "s1",
            entry_action = "enter_s12",
            exit_action = "exit_s12"
        )]
        pub fn s12(&mut self, input: &Event) -> Result {
            match input {
                Event::C => Ok(Transition(State::s211())),
                _ => Ok(Super),
            }
        }

        #[action]
        fn enter_s12(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::s12()), Action::Entry));
        }

        #[action]
        fn exit_s12(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::s12()), Action::Exit));
        }

        /// s1
        #[superstate(superstate = "s", entry_action = "enter_s1", exit_action = "exit_s1")]
        pub fn s1(&mut self, input: &Event) -> Result {
            Ok(Super)
        }

        #[action]
        fn enter_s1(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S1 {}), Action::Entry));
        }

        #[action]
        fn exit_s1(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S1 {}), Action::Exit));
        }

        /// s211
        #[state(
            superstate = "s21",
            entry_action = "enter_s211",
            exit_action = "exit_s211"
        )]
        pub fn s211(&mut self, input: &Event) -> Result {
            Ok(Super)
        }

        #[action]
        fn enter_s211(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::s211()), Action::Entry));
        }

        #[action]
        fn exit_s211(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::s211()), Action::Exit));
        }

        /// s21
        #[superstate(
            superstate = "s2",
            entry_action = "enter_s21",
            exit_action = "exit_s21"
        )]
        pub fn s21(&mut self, input: &Event) -> Result {
            Ok(Super)
        }

        #[action]
        fn enter_s21(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S21 {}), Action::Entry));
        }

        #[action]
        fn exit_s21(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S21 {}), Action::Exit));
        }

        /// s2
        #[superstate(superstate = "s", entry_action = "enter_s2", exit_action = "exit_s2")]
        pub fn s2(&mut self, input: &Event) -> Result {
            match input {
                Event::D => Ok(Transition(State::S11 {})),
                _ => Ok(Super),
            }
        }

        #[action]
        fn enter_s2(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S2 {}), Action::Entry));
        }

        #[action]
        fn exit_s2(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S2 {}), Action::Exit));
        }

        /// s
        #[superstate(entry_action = "enter_s", exit_action = "exit_s")]
        pub fn s(&mut self, input: &Event) -> Result {
            Ok(Handled)
        }

        #[action]
        fn enter_s(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S {}), Action::Entry));
        }

        #[action]
        fn exit_s(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S {}), Action::Exit));
        }
    }

    #[test]
    fn stator_transition() {
        let mut state_machine = StateMachine::new(Foo { path: Vec::new() });

        state_machine.init();
        state_machine.handle(&Event::A);
        state_machine.handle(&Event::B);
        state_machine.handle(&Event::C);
        state_machine.handle(&Event::D);

        let expected_path: [(StateWrapper, Action); 17] = [
            (StateWrapper::Super(Superstate::S {}), Action::Entry),
            (StateWrapper::Super(Superstate::S1 {}), Action::Entry),
            (StateWrapper::Leaf(State::S11 {}), Action::Entry),
            (StateWrapper::Leaf(State::S11 {}), Action::Exit),
            (StateWrapper::Leaf(State::S11 {}), Action::Entry),
            (StateWrapper::Leaf(State::S11 {}), Action::Exit),
            (StateWrapper::Leaf(State::S12 {}), Action::Entry),
            (StateWrapper::Leaf(State::S12 {}), Action::Exit),
            (StateWrapper::Super(Superstate::S1 {}), Action::Exit),
            (StateWrapper::Super(Superstate::S2 {}), Action::Entry),
            (StateWrapper::Super(Superstate::S21 {}), Action::Entry),
            (StateWrapper::Leaf(State::S211 {}), Action::Entry),
            (StateWrapper::Leaf(State::S211 {}), Action::Exit),
            (StateWrapper::Super(Superstate::S21 {}), Action::Exit),
            (StateWrapper::Super(Superstate::S2 {}), Action::Exit),
            (StateWrapper::Super(Superstate::S1 {}), Action::Entry),
            (StateWrapper::Leaf(State::S11 {}), Action::Entry),
        ];

        for (i, expected) in expected_path.iter().enumerate() {
            use StateWrapper::*;
            match (&state_machine.path[i].0, &expected.0) {
                (Super(actual), Super(expected)) if actual == expected => continue,
                (Leaf(actual), Leaf(expected)) if actual == expected => continue,
                _ => panic!(
                    "Transition path at {} is wrong: Actual: {:?}, Expected: {:?}",
                    i, &state_machine.path[i], &expected_path[i]
                ),
            };
        }
    }
}
