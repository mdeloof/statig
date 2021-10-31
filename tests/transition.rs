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

    struct Foo {
        pub state: State,
        pub path: Vec<(State, Action)>,
    }

    impl Stateful for Foo {
        type Event = Event;
        type State = State;

        const INIT_STATE: State = State::S11;

        fn state_mut(&mut self) -> &mut State {
            &mut self.state
        }
    }

    impl Foo {
        /// s11
        pub fn s11(&mut self, event: &Event) -> Response {
            match event {
                Event::A => Response::Transition(State::S11),
                Event::B => Response::Transition(State::S12),
                _ => Response::Parent,
            }
        }

        fn enter_s11(&mut self) {
            self.path.push((State::S11, Action::Entry));
        }

        fn exit_s11(&mut self) {
            self.path.push((State::S11, Action::Exit));
        }

        /// s12
        pub fn s12(&mut self, event: &Event) -> Response {
            match event {
                Event::C => Response::Transition(State::S211),
                _ => Response::Parent,
            }
        }

        fn enter_s12(&mut self) {
            self.path.push((State::S12, Action::Entry));
        }

        fn exit_s12(&mut self) {
            self.path.push((State::S12, Action::Exit));
        }

        /// s1
        pub fn s1(&mut self, event: &Event) -> Response {
            match event {
                _ => Response::Parent,
            }
        }

        fn enter_s1(&mut self) {
            self.path.push((State::S1, Action::Entry));
        }

        fn exit_s1(&mut self) {
            self.path.push((State::S1, Action::Exit));
        }

        /// s211
        pub fn s211(&mut self, event: &Event) -> Response {
            println!("Cool");
            match event {
                _ => Response::Parent,
            }
        }

        fn enter_s211(&mut self) {
            self.path.push((State::S211, Action::Entry));
        }

        fn exit_s211(&mut self) {
            self.path.push((State::S211, Action::Exit));
        }

        /// s21
        pub fn s21(&mut self, event: &Event) -> Response {
            match event {
                _ => Response::Parent,
            }
        }

        fn enter_s21(&mut self) {
            self.path.push((State::S21, Action::Entry));
        }

        fn exit_s21(&mut self) {
            self.path.push((State::S21, Action::Exit));
        }

        /// s2
        pub fn s2(&mut self, event: &Event) -> Response {
            match event {
                Event::D => Response::Transition(State::S11),
                _ => Response::Parent,
            }
        }

        fn enter_s2(&mut self) {
            self.path.push((State::S2, Action::Entry));
        }

        fn exit_s2(&mut self) {
            self.path.push((State::S2, Action::Exit));
        }

        /// s
        pub fn s(&mut self, event: &Event) -> Response {
            match event {
                _ => Response::Handled,
            }
        }

        fn enter_s(&mut self) {
            self.path.push((State::S, Action::Entry));
        }

        fn exit_s(&mut self) {
            self.path.push((State::S, Action::Exit));
        }
    }

    impl fmt::Debug for Foo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "StatorComponent")
        }
    }

    #[derive(Copy, Clone, PartialEq)]
    enum State {
        S,
        S1,
        S11,
        S12,
        S2,
        S21,
        S211,
    }

    impl stateful::State for State {
        type Object = Foo;

        type Event = Event;

        fn state_handler(&self) -> stateful::StateHandler<Self::Object, Self::Event> {
            match self {
                State::S => Foo::s,
                State::S1 => Foo::s1,
                State::S11 => Foo::s11,
                State::S12 => Foo::s12,
                State::S2 => Foo::s2,
                State::S21 => Foo::s21,
                State::S211 => Foo::s211,
            }
        }

        fn parent_state(&self) -> Option<Self> {
            match self {
                State::S => None,
                State::S1 => Some(State::S),
                State::S11 => Some(State::S1),
                State::S12 => Some(State::S1),
                State::S2 => Some(State::S),
                State::S21 => Some(State::S2),
                State::S211 => Some(State::S21),
            }
        }

        fn state_on_enter_handler(&self) -> Option<stateful::StateOnEnterHandler<Self::Object>> {
            match self {
                State::S => Some(Foo::enter_s),
                State::S1 => Some(Foo::enter_s1),
                State::S11 => Some(Foo::enter_s11),
                State::S12 => Some(Foo::enter_s12),
                State::S2 => Some(Foo::enter_s2),
                State::S21 => Some(Foo::enter_s21),
                State::S211 => Some(Foo::enter_s211),
            }
        }

        fn state_on_exit_handler(&self) -> Option<stateful::StateOnExitHandler<Self::Object>> {
            match self {
                State::S => Some(Foo::exit_s),
                State::S1 => Some(Foo::exit_s1),
                State::S11 => Some(Foo::exit_s11),
                State::S12 => Some(Foo::exit_s12),
                State::S2 => Some(Foo::exit_s2),
                State::S21 => Some(Foo::exit_s21),
                State::S211 => Some(Foo::exit_s211),
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

        let expected_path: [(State, Action); 17] = [
            (State::S, Action::Entry),
            (State::S1, Action::Entry),
            (State::S11, Action::Entry),
            (State::S11, Action::Exit),
            (State::S11, Action::Entry),
            (State::S11, Action::Exit),
            (State::S12, Action::Entry),
            (State::S12, Action::Exit),
            (State::S1, Action::Exit),
            (State::S2, Action::Entry),
            (State::S21, Action::Entry),
            (State::S211, Action::Entry),
            (State::S211, Action::Exit),
            (State::S21, Action::Exit),
            (State::S2, Action::Exit),
            (State::S1, Action::Entry),
            (State::S11, Action::Entry),
        ];

        for i in 0..expected_path.len() {
            let actual_state = foo.path[i].0 as usize;
            let expected_state = expected_path[i].0 as usize;
            if actual_state != expected_state {
                panic!("Transition path is wrong.")
            } else {
                continue;
            }
        }
    }
}
