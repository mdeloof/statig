#[cfg(test)]
mod tests {
    use core::cell::RefCell;

    use statig::prelude::*;

    #[derive(Default)]
    struct Counter;

    #[derive(Clone)]
    struct ExternalContext(usize);

    enum Event {
        ButtonPressed,
        TimerElapsed,
    }

    #[state_machine(
        initial = "State::up()",
        event = "(&'a RefCell<&'a mut ExternalContext>, &'a Event)",
        event_pattern = "(external_context, event)"
    )]
    impl Counter {
        #[state]
        fn up(external_context: &RefCell<&mut ExternalContext>, event: &Event) -> Response<State> {
            match event {
                Event::ButtonPressed => {
                    let mut temp = external_context.borrow_mut();
                    temp.0 = temp.0.saturating_add(1);
                    Handled
                }
                Event::TimerElapsed => Transition(State::down()),
            }
        }

        #[state]
        fn down(
            external_context: &RefCell<&mut ExternalContext>,
            event: &Event,
        ) -> Response<State> {
            match event {
                Event::ButtonPressed => {
                    let mut temp = external_context.borrow_mut();
                    temp.0 = temp.0.saturating_sub(1);
                    Handled
                }
                Event::TimerElapsed => Transition(State::up()),
            }
        }
    }

    #[test]
    fn main() {
        let mut blinky = Counter::default().state_machine().init();

        let events = [
            Event::ButtonPressed,
            Event::ButtonPressed,
            Event::ButtonPressed,
        ];

        let mut external_context = ExternalContext(0);
        let refcell_external_context = RefCell::new(&mut external_context);

        for event in &events {
            let composed_event = (&refcell_external_context, event);
            blinky.handle(&composed_event);
        }

        assert_eq!(refcell_external_context.borrow().0, 3);

        let events = [
            Event::TimerElapsed,
            Event::ButtonPressed,
            Event::ButtonPressed,
            Event::ButtonPressed,
        ];

        for event in &events {
            let composed_event = (&refcell_external_context, event);
            blinky.handle(&composed_event);
        }

        assert_eq!(refcell_external_context.borrow().0, 0);
    }
}
