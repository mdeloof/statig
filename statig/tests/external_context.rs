#[cfg(test)]
mod tests {
    use statig::prelude::*;

    #[derive(Default)]
    struct Counter;

    #[derive(Clone)]
    struct ExternalContext(usize);

    enum Event {
        ButtonPressed,
        TimerElapsed,
    }

    #[state_machine(initial = "State::up()")]
    impl Counter {
        #[state]
        fn up(context: &mut ExternalContext, event: &Event) -> Response<State> {
            match event {
                Event::ButtonPressed => {
                    context.0 = context.0.saturating_add(1);
                    Handled
                }
                Event::TimerElapsed => Transition(State::down()),
            }
        }

        #[state]
        fn down(context: &mut ExternalContext, event: &Event) -> Response<State> {
            match event {
                Event::ButtonPressed => {
                    context.0 = context.0.saturating_sub(1);
                    Handled
                }
                Event::TimerElapsed => Transition(State::up()),
            }
        }
    }

    #[test]
    fn main() {
        let mut external_context = ExternalContext(0);

        let mut blinky = Counter::default()
            .uninitialized_state_machine()
            .init_with_context(&mut external_context);

        let events = [
            Event::ButtonPressed,
            Event::ButtonPressed,
            Event::ButtonPressed,
        ];

        for event in &events {
            blinky.handle_with_context(event, &mut external_context);
        }

        assert_eq!(external_context.0, 3);

        let events = [
            Event::TimerElapsed,
            Event::ButtonPressed,
            Event::ButtonPressed,
            Event::ButtonPressed,
        ];

        for event in &events {
            blinky.handle_with_context(event, &mut external_context);
        }

        assert_eq!(external_context.0, 0);
    }
}
