#[cfg(test)]
#[allow(unused)]
mod tests {

    use statig::prelude::*;

    enum Event {
        Tick,
    }

    #[derive(Default)]
    struct FooBar {
        before_dispatch: bool,
        after_dispatch: bool,
        before_transition: bool,
        after_transition: bool,
    }

    #[state_machine(
        initial = "State::foo()",
        before_dispatch = "Self::before_dispatch",
        after_dispatch = "Self::after_dispatch",
        before_transition = "Self::before_transition",
        after_transition = "Self::after_transition"
    )]
    impl FooBar {
        #[state]
        fn foo(event: &Event) -> Response<State> {
            Transition(State::bar())
        }

        #[state]
        fn bar(event: &Event) -> Response<State> {
            Transition(State::foo())
        }
    }

    impl FooBar {
        fn before_dispatch(
            &mut self,
            state_or_superstate: StateOrSuperstate<'_, State, Superstate>,
            event: &Event,
        ) {
            self.before_dispatch = true;
        }

        fn after_dispatch(
            &mut self,
            state_or_superstate: StateOrSuperstate<'_, State, Superstate>,
            event: &Event,
        ) {
            self.after_dispatch = true;
        }

        fn before_transition(&mut self, source: &State, target: &State) {
            self.before_transition = true;
        }

        fn after_transition(&mut self, source: &State, target: &State) {
            self.after_transition = true;
        }
    }

    #[test]
    fn hooks() {
        let mut foo_bar = FooBar::default().state_machine();

        foo_bar.handle(&Event::Tick);

        assert_eq!(foo_bar.inner().before_dispatch, true);
        assert_eq!(foo_bar.inner().after_dispatch, true);
        assert_eq!(foo_bar.inner().before_transition, true);
        assert_eq!(foo_bar.inner().after_transition, true);
    }
}
