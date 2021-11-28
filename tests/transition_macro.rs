mod tests {

    use stateful::state_machine;
    use stateful::Stateful;
    use std::fmt;

    type Response = stateful::Response<Foo>;

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
    pub enum StateWrapper {
        Leaf(State),
        Super(Superstate),
    }

    pub struct Foo {
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

    #[state_machine]
    impl Foo {
        /// s11
        #[state(
            superstate = "S1",
            entry_action = "Foo::enter_s11",
            exit_action = "Foo::exit_s11"
        )]
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
        #[state(
            superstate = "S1",
            entry_action = "Foo::enter_s12",
            exit_action = "Foo::exit_s12"
        )]
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
        #[superstate(
            superstate = "S",
            entry_action = "Foo::enter_s1",
            exit_action = "Foo::exit_s1"
        )]
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
        #[state(
            superstate = "S21",
            entry_action = "Foo::enter_s211",
            exit_action = "Foo::exit_s211"
        )]
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
        #[superstate(
            superstate = "S2",
            entry_action = "Foo::enter_s21",
            exit_action = "Foo::exit_s21"
        )]
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
        #[superstate(
            superstate = "S",
            entry_action = "Foo::enter_s2",
            exit_action = "Foo::exit_s2"
        )]
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
        #[superstate(entry_action = "Foo::enter_s", exit_action = "Foo::exit_s")]
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
