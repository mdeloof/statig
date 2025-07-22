#[cfg(test)]
mod tests {

    use statig::blocking::{self, *};
    use std::fmt;

    type Outcome = statig::Outcome<State>;

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

    impl IntoStateMachine for Foo {
        type State = State;

        type Superstate<'sub> = Superstate;

        type Event<'evt> = Event;

        type Context<'ctx> = ();

        type Response = ();

        fn initial() -> Self::State {
            State::S11
        }

        fn default_response() -> Self::Response {
            ()
        }
    }

    impl blocking::State<Foo> for State {
        fn call_handler(
            &mut self,
            shared_storage: &mut Foo,
            event: &Event,
            _: &mut (),
        ) -> statig::Outcome<Self> {
            match self {
                State::S211 {} => Foo::s211(shared_storage, event),
                State::S11 {} => Foo::s11(shared_storage, event),
                State::S12 {} => Foo::s12(shared_storage, event),
            }
        }

        fn call_entry_action(&mut self, shared_storage: &mut Foo, _: &mut ()) {
            match self {
                State::S211 {} => Foo::enter_s211(shared_storage),
                State::S11 {} => Foo::enter_s11(shared_storage),
                State::S12 {} => Foo::enter_s12(shared_storage),
            }
        }

        fn call_exit_action(&mut self, shared_storage: &mut Foo, _: &mut ()) {
            match self {
                State::S211 {} => Foo::exit_s211(shared_storage),
                State::S11 {} => Foo::exit_s11(shared_storage),
                State::S12 {} => Foo::exit_s12(shared_storage),
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

    impl<'sub> blocking::Superstate<Foo> for Superstate
    where
        Self: 'sub,
    {
        fn call_handler(
            &mut self,
            shared_storage: &mut Foo,
            event: &Event,
            _: &mut (),
        ) -> statig::Outcome<State>
        where
            Self: Sized,
        {
            match self {
                Superstate::S21 {} => Foo::s21(shared_storage, event),
                Superstate::S {} => Foo::s(shared_storage, event),
                Superstate::S2 {} => Foo::s2(shared_storage, event),
                Superstate::S1 {} => Foo::s1(shared_storage, event),
            }
        }

        fn call_entry_action(&mut self, shared_storage: &mut Foo, _: &mut ()) {
            match self {
                Superstate::S21 {} => Foo::enter_s21(shared_storage),
                Superstate::S {} => Foo::enter_s(shared_storage),
                Superstate::S2 {} => Foo::enter_s2(shared_storage),
                Superstate::S1 {} => Foo::enter_s1(shared_storage),
            }
        }

        fn call_exit_action(&mut self, shared_storage: &mut Foo, _: &mut ()) {
            match self {
                Superstate::S21 {} => Foo::exit_s21(shared_storage),
                Superstate::S {} => Foo::exit_s(shared_storage),
                Superstate::S2 {} => Foo::exit_s2(shared_storage),
                Superstate::S1 {} => Foo::exit_s1(shared_storage),
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
        pub fn s11(&mut self, event: &Event) -> Outcome {
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
        pub fn s12(&mut self, event: &Event) -> Outcome {
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
        pub fn s1(&mut self, event: &Event) -> Outcome {
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
        pub fn s211(&mut self, event: &Event) -> Outcome {
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
        pub fn s21(&mut self, event: &Event) -> Outcome {
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
        pub fn s2(&mut self, event: &Event) -> Outcome {
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
        pub fn s(&mut self, event: &Event) -> Outcome {
            Handled(())
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
        let mut state_machine = Foo::default().uninitialized_state_machine().init();

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
