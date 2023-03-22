#[cfg(test)]
#[cfg(feature = "async")]
mod tests {
    use core::future::Future;
    use core::pin::Pin;
    use statig::awaitable::{self, *};
    use std::fmt;

    type Response = statig::Response<State>;

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

        const INITIAL: State = State::S11;
    }

    impl awaitable::State<Foo> for State {
        fn call_handler<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut Foo,
            event: &'fut <Foo as IntoStateMachine>::Event<'_>,
            _: &'fut mut <Foo as IntoStateMachine>::Context<'_>,
        ) -> Pin<Box<dyn Future<Output = statig::Response<Self>> + 'fut + Send>> {
            Box::pin(async move {
                match self {
                    State::S211 {} => Foo::s211(shared_storage, event).await,
                    State::S11 {} => Foo::s11(shared_storage, event).await,
                    State::S12 {} => Foo::s12(shared_storage, event).await,
                }
            })
        }

        fn call_entry_action<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut Foo,
            _: &'fut mut <Foo as IntoStateMachine>::Context<'_>,
        ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
            Box::pin(async move {
                match self {
                    State::S211 {} => Foo::enter_s211(shared_storage).await,
                    State::S11 {} => Foo::enter_s11(shared_storage).await,
                    State::S12 {} => Foo::enter_s12(shared_storage).await,
                }
            })
        }

        fn call_exit_action<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut Foo,
            _: &'fut mut <Foo as IntoStateMachine>::Context<'_>,
        ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
            Box::pin(async move {
                match self {
                    State::S211 {} => Foo::exit_s211(shared_storage).await,
                    State::S11 {} => Foo::exit_s11(shared_storage).await,
                    State::S12 {} => Foo::exit_s12(shared_storage).await,
                }
            })
        }

        fn superstate(&mut self) -> Option<Superstate> {
            match self {
                State::S211 {} => Some(Superstate::S21 {}),
                State::S11 {} => Some(Superstate::S1 {}),
                State::S12 {} => Some(Superstate::S1 {}),
            }
        }
    }

    impl awaitable::Superstate<Foo> for Superstate {
        fn call_handler<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut Foo,
            event: &'fut <Foo as IntoStateMachine>::Event<'_>,
            _: &'fut mut <Foo as IntoStateMachine>::Context<'_>,
        ) -> Pin<Box<dyn Future<Output = statig::Response<State>> + 'fut + Send>>
        where
            Self: Sized,
        {
            Box::pin(async move {
                match self {
                    Superstate::S21 {} => Foo::s21(shared_storage, event).await,
                    Superstate::S {} => Foo::s(shared_storage, event).await,
                    Superstate::S2 {} => Foo::s2(shared_storage, event).await,
                    Superstate::S1 {} => Foo::s1(shared_storage, event).await,
                }
            })
        }

        fn call_entry_action<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut Foo,
            _: &'fut mut <Foo as IntoStateMachine>::Context<'_>,
        ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
            Box::pin(async move {
                match self {
                    Superstate::S21 {} => Foo::enter_s21(shared_storage).await,
                    Superstate::S {} => Foo::enter_s(shared_storage).await,
                    Superstate::S2 {} => Foo::enter_s2(shared_storage).await,
                    Superstate::S1 {} => Foo::enter_s1(shared_storage).await,
                }
            })
        }

        fn call_exit_action<'fut>(
            &'fut mut self,
            shared_storage: &'fut mut Foo,
            _: &'fut mut <Foo as IntoStateMachine>::Context<'_>,
        ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
            Box::pin(async move {
                match self {
                    Superstate::S21 {} => Foo::exit_s21(shared_storage).await,
                    Superstate::S {} => Foo::exit_s(shared_storage).await,
                    Superstate::S2 {} => Foo::exit_s2(shared_storage).await,
                    Superstate::S1 {} => Foo::exit_s1(shared_storage).await,
                }
            })
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
        pub async fn s11(&mut self, event: &Event) -> Response {
            match event {
                Event::A => Transition(State::S11),
                Event::B => Transition(State::S12),
                _ => Super,
            }
        }

        async fn enter_s11(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S11), Action::Entry));
        }

        async fn exit_s11(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S11), Action::Exit));
        }

        /// s12
        async fn s12(&mut self, event: &Event) -> Response {
            match event {
                Event::C => Transition(State::S211),
                _ => Super,
            }
        }

        async fn enter_s12(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S12), Action::Entry));
        }

        async fn exit_s12(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S12), Action::Exit));
        }

        /// s1
        #[allow(unused)]
        async fn s1(&mut self, event: &Event) -> Response {
            Super
        }

        async fn enter_s1(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S1), Action::Entry));
        }

        async fn exit_s1(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S1), Action::Exit));
        }

        /// s211
        #[allow(unused)]
        async fn s211(&mut self, event: &Event) -> Response {
            Super
        }

        async fn enter_s211(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S211), Action::Entry));
        }

        async fn exit_s211(&mut self) {
            self.path
                .push((StateWrapper::Leaf(State::S211), Action::Exit));
        }

        /// s21
        #[allow(unused)]
        pub async fn s21(&mut self, event: &Event) -> Response {
            Super
        }

        async fn enter_s21(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S21), Action::Entry));
        }

        async fn exit_s21(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S21), Action::Exit));
        }

        /// s2
        pub async fn s2(&mut self, event: &Event) -> Response {
            match event {
                Event::D => Transition(State::S11),
                _ => Super,
            }
        }

        async fn enter_s2(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S2), Action::Entry));
        }

        async fn exit_s2(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S2), Action::Exit));
        }

        /// s
        #[allow(unused)]
        async fn s(&mut self, event: &Event) -> Response {
            Handled
        }

        async fn enter_s(&mut self) {
            self.path
                .push((StateWrapper::Super(Superstate::S), Action::Entry));
        }

        async fn exit_s(&mut self) {
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
        let task = async {
            let context = &mut ();
            let mut state_machine = Foo::default()
                .uninitialized_state_machine()
                .init_with_context(context)
                .await;

            state_machine.handle_with_context(&Event::A, context).await;
            state_machine.handle_with_context(&Event::B, context).await;
            state_machine.handle_with_context(&Event::C, context).await;
            state_machine.handle_with_context(&Event::D, context).await;

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
        };
        futures::executor::block_on(task);
    }
}
