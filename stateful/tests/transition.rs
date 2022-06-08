#![feature(generic_associated_types)]

#[cfg(test)]
mod tests {

    use stateful::Response::*;
    use stateful::{StateMachine, Stateful};
    use std::fmt;

    type Result = stateful::Result<State>;

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

    struct Foo {
        pub path: Vec<(StateWrapper, Action)>,
    }

    impl Stateful for Foo {
        type State = State;

        type Input = Event;

        const INIT_STATE: State = State::S11;
    }

    impl Foo {
        /// s11
        pub fn s11(&mut self, event: &Event) -> Result {
            match event {
                Event::A => Ok(Transition(State::S11)),
                Event::B => Ok(Transition(State::S12)),
                _ => Ok(Super),
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
        pub fn s12(&mut self, event: &Event) -> Result {
            match event {
                Event::C => Ok(Transition(State::S211)),
                _ => Ok(Super),
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
        pub fn s1(&mut self, event: &Event) -> Result {
            Ok(Super)
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
        pub fn s211(&mut self, event: &Event) -> Result {
            Ok(Super)
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
        pub fn s21(&mut self, event: &Event) -> Result {
            Ok(Super)
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
        pub fn s2(&mut self, event: &Event) -> Result {
            match event {
                Event::D => Ok(Transition(State::S11)),
                _ => Ok(Super),
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
        pub fn s(&mut self, event: &Event) -> Result {
            Ok(Handled)
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

    #[derive(Copy, Clone, PartialEq, Debug)]
    enum State {
        S11,
        S12,
        S211,
    }

    impl stateful::State for State {
        type Superstate<'a> = Superstate;

        type Object = Foo;

        fn call_handler(
            &mut self,
            object: &mut Self::Object,
            input: &<Self::Object as Stateful>::Input,
        ) -> stateful::Result<Self>
        where
            Self: Sized,
        {
            match self {
                State::S211 {} => Foo::s211(object, input),
                State::S11 {} => Foo::s11(object, input),
                State::S12 {} => Foo::s12(object, input),
            }
        }

        fn call_entry_action(&mut self, object: &mut Self::Object) {
            match self {
                State::S211 {} => Foo::enter_s211(object),
                State::S11 {} => Foo::enter_s11(object),
                State::S12 {} => Foo::enter_s12(object),
            }
        }

        fn call_exit_action(&mut self, object: &mut Self::Object) {
            match self {
                State::S211 {} => Foo::exit_s211(object),
                State::S11 {} => Foo::exit_s11(object),
                State::S12 {} => Foo::exit_s12(object),
            }
        }

        fn superstate(&mut self) -> Option<Self::Superstate<'_>> {
            match self {
                State::S211 {} => Some(Superstate::S21 {}),
                State::S11 {} => Some(Superstate::S1 {}),
                State::S12 {} => Some(Superstate::S1 {}),
            }
        }

        fn same_state(&self, state: &Self) -> bool {
            #[allow(clippy::match_like_matches_macro)]
            match (self, state) {
                (State::S211 { .. }, State::S211 { .. }) => true,
                (State::S11 { .. }, State::S11 { .. }) => true,
                (State::S12 { .. }, State::S12 { .. }) => true,
                _ => false,
            }
        }

        fn depth(&mut self) -> usize {
            match self {
                State::S11 => 3,
                State::S12 => 3,
                State::S211 => 4,
            }
        }
    }

    #[derive(Copy, Clone, PartialEq, Debug)]
    enum Superstate {
        S,
        S1,
        S2,
        S21,
    }

    impl<'a> stateful::Superstate for Superstate
    where
        Self: 'a,
    {
        type State = State;

        fn call_handler(
            &mut self,
            object: &mut <Self::State as stateful::State>::Object,
            input: &<<Self::State as stateful::State>::Object as Stateful>::Input,
        ) -> stateful::Result<Self::State>
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

        fn call_entry_action(&mut self, object: &mut <Self::State as stateful::State>::Object) {
            match self {
                Superstate::S21 {} => Foo::enter_s21(object),
                Superstate::S {} => Foo::enter_s(object),
                Superstate::S2 {} => Foo::enter_s2(object),
                Superstate::S1 {} => Foo::enter_s1(object),
            }
        }

        fn call_exit_action(&mut self, object: &mut <Self::State as stateful::State>::Object) {
            match self {
                Superstate::S21 {} => Foo::exit_s21(object),
                Superstate::S {} => Foo::exit_s(object),
                Superstate::S2 {} => Foo::exit_s2(object),
                Superstate::S1 {} => Foo::exit_s1(object),
            }
        }

        fn superstate(&mut self) -> Option<<Self::State as stateful::State>::Superstate<'_>> {
            match self {
                Superstate::S21 {} => Some(Superstate::S2 {}),
                Superstate::S {} => None,
                Superstate::S2 {} => Some(Superstate::S {}),
                Superstate::S1 {} => Some(Superstate::S {}),
            }
        }

        fn same_state(&self, state: &<Self::State as stateful::State>::Superstate<'_>) -> bool {
            #[allow(clippy::match_like_matches_macro)]
            match (self, state) {
                (Superstate::S21 { .. }, Superstate::S21 { .. }) => true,
                (Superstate::S { .. }, Superstate::S { .. }) => true,
                (Superstate::S2 { .. }, Superstate::S2 { .. }) => true,
                (Superstate::S1 { .. }, Superstate::S1 { .. }) => true,
                _ => false,
            }
        }

        fn depth(&mut self) -> usize {
            match self {
                Superstate::S => 1,
                Superstate::S1 => 2,
                Superstate::S2 => 2,
                Superstate::S21 => 3,
            }
        }
    }

    #[test]
    fn test_transition_path() {
        let mut state_machine = StateMachine::new(Foo { path: Vec::new() });

        state_machine.init();

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
