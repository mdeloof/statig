#[cfg(test)]
mod tests {
    mod module_a {
        #![allow(unused)]

        use std::convert::identity;

        use statig::prelude::*;

        #[allow(dead_code)]
        enum Event {
            A,
        }

        #[allow(dead_code)]
        struct Context;

        struct MyStateMachine;

        #[state_machine(
            initial = "MyState::bar()",
            state(name = "MyState", derive(PartialEq, Eq)),
            superstate(name = "MySuperstate", derive(Clone)),
            event_identifier = "my_event",
            context_identifier = "my_context",
            visibility = "pub(crate)"
        )]
        impl MyStateMachine {
            #[state(superstate = "foo")]
            fn bar(my_event: &Event, my_context: &mut Context) -> Outcome<MyState> {
                match my_event {
                    Event::A => Handled(()),
                }
            }

            #[superstate]
            fn foo(my_event: &Event, my_context: &mut Context) -> Outcome<MyState> {
                match my_event {
                    Event::A => Handled(()),
                }
            }
        }
    }

    // Check that visibility configuration is correctly set.
    #[allow(unused)]
    use module_a::MyState;
    use module_a::MySuperstate;

    // Check that derive traits are correctly applied.
    #[allow(unused)]
    fn compare() {
        _ = MyState::Bar {} == MyState::Bar {};
        _ = MySuperstate::Foo {}.clone();
    }
}
