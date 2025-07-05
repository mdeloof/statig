#[cfg(test)]
#[cfg(feature = "async")]
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
        initial = State::foo(),
        before_dispatch = Self::before_dispatch,
        after_dispatch = Self::after_dispatch,
        before_transition = Self::before_transition,
        after_transition = Self::after_transition
    )]
    impl FooBar {
        #[state]
        async fn foo(event: &Event) -> Outcome<State> {
            Transition(State::bar())
        }

        #[state]
        async fn bar(event: &Event) -> Outcome<State> {
            Transition(State::foo())
        }
    }

    impl FooBar {
        async fn before_dispatch(
            &mut self,
            state_or_superstate: StateOrSuperstate<'_, State, Superstate>,
            event: &Event,
        ) {
            self.before_dispatch = true;
        }

        async fn after_dispatch(
            &mut self,
            state_or_superstate: StateOrSuperstate<'_, State, Superstate>,
            event: &Event,
            _context: &mut (),
        ) {
            self.after_dispatch = true;
        }

        async fn before_transition(&mut self, source: &State, target: &State) {
            self.before_transition = true;
        }

        async fn after_transition(&mut self, source: &State, target: &State, _context: &mut ()) {
            self.after_transition = true;
        }
    }

    #[test]
    fn async_hooks() {
        let mut foo_bar = FooBar::default().state_machine();

        futures::executor::block_on(async {
            foo_bar.handle(&Event::Tick).await;
        });

        assert_eq!(foo_bar.inner().before_dispatch, true);
        assert_eq!(foo_bar.inner().after_dispatch, true);
        assert_eq!(foo_bar.inner().before_transition, true);
        assert_eq!(foo_bar.inner().after_transition, true);
    }
}
