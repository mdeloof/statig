#[cfg(test)]
mod tests {
    #![allow(unused)]

    use statig::blocking::*;

    pub enum Event {
        Transition,
        TestSuperAck,
        TestSuperData,
        TestNak,
    }

    #[derive(Debug, PartialEq)]
    pub enum Response {
        Ack,
        Data(u8),
    }

    impl core::default::Default for Response {
        fn default() -> Self {
            Response::Ack
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum Nak {
        Reason,
        AnotherReason,
    }

    pub struct Foo;

    #[state_machine(initial = "State::bar()")]
    impl Foo {
        #[state(superstate = "baz")]
        fn bar(event: &Event) -> Outcome<State, Result<Response, Nak>> {
            match event {
                Event::Transition => Transition(State::qux()),
                Event::TestSuperAck => Super,
                Event::TestSuperData => Super,
                Event::TestNak => Handled(Err(Nak::Reason)),
            }
        }

        #[superstate]
        fn baz(event: &Event) -> Outcome<State, Result<Response, Nak>> {
            match event {
                Event::TestSuperAck => Handled(Ok(Response::Ack)),
                Event::TestSuperData => Handled(Ok(Response::Data(42))),
                _ => Handled(Ok(Response::default())),
            }
        }

        #[state]
        fn qux(event: &Event) -> Outcome<State, Result<Response, Nak>> {
            Handled(Ok(Response::default()))
        }

    }

    #[test]
    fn transition_test() {
        let mut foo = Foo.state_machine();
        
        let response = foo.handle(&Event::Transition);
        assert_eq!(response, Ok(Response::Ack));
    }

    #[test]
    fn super_ack_test() {
        let mut foo = Foo.state_machine();
        
        let response = foo.handle(&Event::TestSuperAck);
        assert_eq!(response, Ok(Response::Ack));
    }

    #[test]
    fn super_nak_data() {
        let mut foo = Foo.state_machine();

        let response = foo.handle(&Event::TestSuperData);
        assert_eq!(response, Ok(Response::Data(42)));
    }

    #[test]
    fn nak_test() {
        let mut foo = Foo.state_machine();

        let response = foo.handle(&Event::TestNak);
        assert_eq!(response, Err(Nak::Reason));
    }
}
