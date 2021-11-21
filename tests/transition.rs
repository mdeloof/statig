#[cfg(test)]
mod tests {

    use stateful::Stateful;
    use std::fmt;

    type Response = stateful::Response<Foo>;

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
        pub state: State,
        pub path: Vec<(StateWrapper, Action)>,
    }

    impl Stateful for Foo {
        type State = State;

        const INIT_STATE: State = State::S11;

        fn state_mut(&mut self) -> &mut State {
            &mut self.state
        }

        fn on_transition(
            &mut self,
            source: &State,
            exit_path: &[Superstate],
            entry_path: &[Superstate],
            target: &State,
        ) {
            println!("source state: {:?}", source);
            println!("exit path: {:?}", exit_path);
            println!("entry path: {:?}", entry_path);
            println!("target state: {:?}", target);
        }
    }

    impl Foo {
        /// s11
        pub fn s11(&mut self, event: &Event) -> Response {
            match event {
                Event::A => Response::Transition(State::S11),
                Event::B => Response::Transition(State::S12),
                _ => Response::Super,
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
                Event::C => Response::Transition(State::S211),
                _ => Response::Super,
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
        pub fn s1(&mut self, event: &Event) -> Response {
            match event {
                _ => Response::Super,
            }
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
        pub fn s211(&mut self, event: &Event) -> Response {
            println!("Cool");
            match event {
                _ => Response::Super,
            }
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
        pub fn s21(&mut self, event: &Event) -> Response {
            match event {
                _ => Response::Super,
            }
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
                Event::D => Response::Transition(State::S11),
                _ => Response::Super,
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
        pub fn s(&mut self, event: &Event) -> Response {
            match event {
                _ => Response::Handled,
            }
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
        type Object = Foo;
        type Event = Event;
        type Superstate = Superstate;

        fn handler(&self) -> stateful::Handler<Self::Object, Self::Event> {
            match self {
                State::S11 => Foo::s11,
                State::S12 => Foo::s12,
                State::S211 => Foo::s211,
            }
        }

        fn superstate(&self) -> Option<Self::Superstate> {
            match self {
                State::S11 => Some(Self::Superstate::S1),
                State::S12 => Some(Self::Superstate::S1),
                State::S211 => Some(Self::Superstate::S21),
            }
        }

        fn entry_action(&self) -> Option<stateful::Action<Self::Object>> {
            match self {
                State::S11 => Some(Foo::enter_s11),
                State::S12 => Some(Foo::enter_s12),
                State::S211 => Some(Foo::enter_s211),
            }
        }

        fn exit_action(&self) -> Option<stateful::Action<Self::Object>> {
            match self {
                State::S11 => Some(Foo::exit_s11),
                State::S12 => Some(Foo::exit_s12),
                State::S211 => Some(Foo::exit_s211),
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

    impl stateful::Superstate for Superstate {
        type Object = Foo;
        type Event = Event;

        fn handler(&self) -> stateful::Handler<Self::Object, Self::Event> {
            match self {
                Superstate::S => Foo::s,
                Superstate::S1 => Foo::s1,
                Superstate::S2 => Foo::s2,
                Superstate::S21 => Foo::s21,
            }
        }

        fn superstate(&self) -> Option<Self> {
            match self {
                Superstate::S => None,
                Superstate::S1 => Some(Superstate::S),
                Superstate::S2 => Some(Superstate::S),
                Superstate::S21 => Some(Superstate::S2),
            }
        }

        fn entry_action(&self) -> Option<stateful::Action<Self::Object>> {
            match self {
                Superstate::S => Some(Foo::enter_s),
                Superstate::S1 => Some(Foo::enter_s1),
                Superstate::S2 => Some(Foo::enter_s2),
                Superstate::S21 => Some(Foo::enter_s21),
            }
        }

        fn exit_action(&self) -> Option<stateful::Action<Self::Object>> {
            match self {
                Superstate::S => Some(Foo::exit_s),
                Superstate::S1 => Some(Foo::exit_s1),
                Superstate::S2 => Some(Foo::exit_s2),
                Superstate::S21 => Some(Foo::exit_s21),
            }
        }
    }

    #[test]
    fn stator_transition() {
        let mut foo = Foo {
            state: State::S11,
            path: Vec::new(),
        };

        foo.init();
        foo.handle(&Event::A);
        foo.handle(&Event::B);
        foo.handle(&Event::C);
        foo.handle(&Event::D);

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

        dbg!(&foo.path);

        for i in 0..expected_path.len() {
            use StateWrapper::*;
            match (&foo.path[i].0, &expected_path[i].0) {
                (Super(actual), Super(expected)) if actual == expected => continue,
                (Leaf(actual), Leaf(expected)) if actual == expected => continue,
                _ => panic!(
                    "Transition path at {} is wrong: Actual: {:?}, Expected: {:?}",
                    i, &foo.path[i], &expected_path[i]
                ),
            };
        }
    }
}
