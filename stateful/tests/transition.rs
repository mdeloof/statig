#![feature(generic_associated_types)]

#[cfg(test)]
mod tests {

    use stateful::prelude::*;
    use std::fmt;

    type Response = stateful::Response<State>;

    #[derive(Clone)]
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

    #[derive(Copy, Clone, PartialEq, Debug)]
    enum State {
        S11,
        S12,
        S211,
    }

    #[derive(Copy, Clone, PartialEq, Debug)]
    enum Superstate {
        S,
        S1,
        S2,
        S21,
    }

    impl StateMachine for Foo {
        type State = State;

        type Superstate<'a> = Superstate;

        type Input = Event;

        type Context = Self;

        const INIT_STATE: State = State::S11;
    }

    impl stateful::State<Foo> for State {
        fn call_handler(&mut self, object: &mut Foo, input: &Event) -> stateful::Response<Self> {
            match self {
                State::S211 {} => Foo::s211(object, input),
                State::S11 {} => Foo::s11(object, input),
                State::S12 {} => Foo::s12(object, input),
            }
        }

        fn call_entry_action(&mut self, object: &mut Foo) {
            match self {
                State::S211 {} => Foo::enter_s211(object),
                State::S11 {} => Foo::enter_s11(object),
                State::S12 {} => Foo::enter_s12(object),
            }
        }

        fn call_exit_action(&mut self, object: &mut Foo) {
            match self {
                State::S211 {} => Foo::exit_s211(object),
                State::S11 {} => Foo::exit_s11(object),
                State::S12 {} => Foo::exit_s12(object),
            }
        }

        fn superstate(&mut self) -> Option<Superstate> {
            match self {
                State::S211 {} => Some(Superstate::S21 {}),
                State::S11 {} => Some(Superstate::S1 {}),
                State::S12 {} => Some(Superstate::S1 {}),
            }
        }
    }

    impl stateful::Superstate<Foo> for Superstate {
        fn call_handler(&mut self, object: &mut Foo, input: &Event) -> stateful::Response<State>
        where
            Self: Sized,
        {
            match self {
                Superstate::S21 {} => Foo::s21(object, input),
                Superstate::S {} => Foo::s(object, input),
                Superstate::S2 {} => Foo::s2(object, input),
                Superstate::S1 {} => Foo::s1(object, input),
            }
        }

        fn call_entry_action(&mut self, object: &mut Foo) {
            match self {
                Superstate::S21 {} => Foo::enter_s21(object),
                Superstate::S {} => Foo::enter_s(object),
                Superstate::S2 {} => Foo::enter_s2(object),
                Superstate::S1 {} => Foo::enter_s1(object),
            }
        }

        fn call_exit_action(&mut self, object: &mut Foo) {
            match self {
                Superstate::S21 {} => Foo::exit_s21(object),
                Superstate::S {} => Foo::exit_s(object),
                Superstate::S2 {} => Foo::exit_s2(object),
                Superstate::S1 {} => Foo::exit_s1(object),
            }
        }

        fn superstate(&mut self) -> Option<Superstate> {
            match self {
                Superstate::S21 {} => Some(Superstate::S2 {}),
                Superstate::S {} => None,
                Superstate::S2 {} => Some(Superstate::S {}),
                Superstate::S1 {} => Some(Superstate::S {}),
            }
        }
    }

    impl Foo {
        /// s11
        pub fn s11(&mut self, event: &Event) -> Response {
            match event {
                Event::A => Transition(State::S11),
                Event::B => Transition(State::S12),
                _ => Super,
            }
        }

        fn enter_s11(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S11), Action::Entry));
        }

        fn exit_s11(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S11), Action::Exit));
        }

        /// s12
        pub fn s12(&mut self, event: &Event) -> Response {
            match event {
                Event::C => Transition(State::S211),
                _ => Super,
            }
        }

        fn enter_s12(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S12), Action::Entry));
        }

        fn exit_s12(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S12), Action::Exit));
        }

        /// s1
        #[allow(unused)]
        pub fn s1(&mut self, event: &Event) -> Response {
            Super
        }

        fn enter_s1(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S1), Action::Entry));
        }

        fn exit_s1(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S1), Action::Exit));
        }

        /// s211
        #[allow(unused)]
        pub fn s211(&mut self, event: &Event) -> Response {
            Super
        }

        fn enter_s211(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S211), Action::Entry));
        }

        fn exit_s211(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S211), Action::Exit));
        }

        /// s21
        #[allow(unused)]
        pub fn s21(&mut self, event: &Event) -> Response {
            Super
        }

        fn enter_s21(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S21), Action::Entry));
        }

        fn exit_s21(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S21), Action::Exit));
        }

        /// s2
        pub fn s2(&mut self, event: &Event) -> Response {
            match event {
                Event::D => Transition(State::S11),
                _ => Super,
            }
        }

        fn enter_s2(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S2), Action::Entry));
        }

        fn exit_s2(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S2), Action::Exit));
        }

        /// s
        #[allow(unused)]
        pub fn s(&mut self, event: &Event) -> Response {
            Handled
        }

        fn enter_s(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S), Action::Entry));
        }

        fn exit_s(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S), Action::Exit));
        }
    }

    impl fmt::Debug for Foo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "StatorComponent")
        }
    }

    #[test]
    fn test_transition_path() {
        let mut state_machine = Foo::state_machine().init();

        state_machine.handle(&Event::A);
        state_machine.handle(&Event::B);
        state_machine.handle(&Event::C);
        state_machine.handle(&Event::D);

        let expected_path: [(StateWrapper, Action); 17] = [
            (StateWrapper::Super(Superstate::S), Action::Entry),
            (StateWrapper::Super(Superstate::S1), Action::Entry),
            (StateWrapper::Leaf(State::S11), Action::Entry),
            (StateWrapper::Leaf(State::S11), Action::Exit),
            (StateWrapper::Leaf(State::S11), Action::Entry),
            (StateWrapper::Leaf(State::S11), Action::Exit),
            (StateWrapper::Leaf(State::S12), Action::Entry),
            (StateWrapper::Leaf(State::S12), Action::Exit),
            (StateWrapper::Super(Superstate::S1), Action::Exit),
            (StateWrapper::Super(Superstate::S2), Action::Entry),
            (StateWrapper::Super(Superstate::S21), Action::Entry),
            (StateWrapper::Leaf(State::S211), Action::Entry),
            (StateWrapper::Leaf(State::S211), Action::Exit),
            (StateWrapper::Super(Superstate::S21), Action::Exit),
            (StateWrapper::Super(Superstate::S2), Action::Exit),
            (StateWrapper::Super(Superstate::S1), Action::Entry),
            (StateWrapper::Leaf(State::S11), Action::Entry),
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
