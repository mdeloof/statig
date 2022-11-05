#[cfg(test)]
mod tests {

    use statig::prelude::*;
    use std::fmt;

    type Response = statig::Response<State>;

    #[derive(Clone, Debug)]
    enum Event {
        A,
        B,
        C,
        D,
    }

    #[derive(Copy, Clone, Debug)]
    enum Action {
        Entry,
        Exit,
    }

    #[derive(Debug)]
    enum StateWrapper {
        Leaf(State),
        Super(Superstate),
    }

    #[derive(Default)]
    struct Foo {
        pub path: Vec<(StateWrapper, Action)>,
    }

    impl StateMachine for Foo {
        type State = State;

        type Superstate<'a> = Superstate;

        type Event = Event;

        const INIT_STATE: State = State::s11();

        fn on_dispatch(
            &mut self,
            state: statig::StateOrSuperstate<'_, '_, Self>,
            event: &Self::Event,
        ) {
            println!("dispatching event {:?} to {:?}", event, state);
        }
    }

    #[state_machine(
        event = "event",
        state(derive(Eq, PartialEq, Debug)),
        superstate(derive(Eq, PartialEq, Debug))
    )]
    impl Foo {
        /// s11
        #[state(
            superstate = "s1",
            entry_action = "enter_s11",
            exit_action = "exit_s11"
        )]
        pub fn s11(&mut self, event: &Event) -> Response {
            match event {
                Event::A => Transition(State::s11()),
                Event::B => Transition(State::s12()),
                _ => Super,
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
        pub fn s12(&mut self, event: &Event) -> Response {
            match event {
                Event::C => Transition(State::s211()),
                _ => Super,
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
        #[allow(unused)]
        #[superstate(superstate = "s", entry_action = "enter_s1", exit_action = "exit_s1")]
        pub fn s1(&mut self, event: &Event) -> Response {
            Super
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
        #[allow(unused)]
        #[state(
            superstate = "s21",
            entry_action = "enter_s211",
            exit_action = "exit_s211"
        )]
        pub fn s211(&mut self, event: &Event) -> Response {
            Super
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
        #[allow(unused)]
        #[superstate(
            superstate = "s2",
            entry_action = "enter_s21",
            exit_action = "exit_s21"
        )]
        pub fn s21(&mut self, event: &Event) -> Response {
            Super
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
        pub fn s2(&mut self, event: &Event) -> Response {
            match event {
                Event::D => Transition(State::s11()),
                _ => Super,
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
        #[allow(unused)]
        #[superstate(entry_action = "enter_s", exit_action = "exit_s")]
        pub fn s(&mut self, event: &Event) -> Response {
            Handled
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

    impl fmt::Debug for Foo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "StatorComponent")
        }
    }

    #[test]
    fn test_transition_path() {
        let mut state_machine = Foo::default().state_machine().init();

        state_machine.handle(&Event::A);
        state_machine.handle(&Event::B);
        state_machine.handle(&Event::C);
        state_machine.handle(&Event::D);

        let expected_path: [(StateWrapper, Action); 17] = [
            (StateWrapper::Super(Superstate::S {}), Action::Entry),
            (StateWrapper::Super(Superstate::S1 {}), Action::Entry),
            (StateWrapper::Leaf(State::s11()), Action::Entry),
            (StateWrapper::Leaf(State::s11()), Action::Exit),
            (StateWrapper::Leaf(State::s11()), Action::Entry),
            (StateWrapper::Leaf(State::s11()), Action::Exit),
            (StateWrapper::Leaf(State::s12()), Action::Entry),
            (StateWrapper::Leaf(State::s12()), Action::Exit),
            (StateWrapper::Super(Superstate::S1 {}), Action::Exit),
            (StateWrapper::Super(Superstate::S2 {}), Action::Entry),
            (StateWrapper::Super(Superstate::S21 {}), Action::Entry),
            (StateWrapper::Leaf(State::s211()), Action::Entry),
            (StateWrapper::Leaf(State::s211()), Action::Exit),
            (StateWrapper::Super(Superstate::S21 {}), Action::Exit),
            (StateWrapper::Super(Superstate::S2 {}), Action::Exit),
            (StateWrapper::Super(Superstate::S1 {}), Action::Entry),
            (StateWrapper::Leaf(State::s11()), Action::Entry),
        ];

        for (i, expected) in expected_path.iter().enumerate() {
            use StateWrapper::*;
            match (&expected.0, &state_machine.path[i].0) {
                (Super(expected), Super(actual)) if actual == expected => continue,
                (Leaf(expected), Leaf(actual)) if actual == expected => continue,
                _ => panic!(
                    "Transition path at {} is wrong: Actual: {:?}, Expected: {:?}",
                    i, &state_machine.path[i], &expected_path[i]
                ),
            };
        }
    }
}
