#[cfg(test)]
mod tests {
    #![allow(unused)]

    use statig::blocking::*;

    pub enum Event {
        Transition,
        TestSuperAck,
        TestSuperNak
    }

    #[derive(Debug, PartialEq)]
    pub enum Response {
        ResponseOk,
        ResponseNak,
    }

    impl core::default::Default for Response {
        fn default() -> Self {
            Response::ResponseOk
        }
    }

    pub struct Foo;

    #[state_machine(initial = "State::bar()")]
    impl Foo {
        #[state(superstate = "baz")]
        fn bar(event: &Event) -> Outcome<State, Response> {
            match event {
                Event::Transition => Transition(State::qux()),
                Event::TestSuperAck => Super,
                Event::TestSuperNak => Super,
            }
        }

        #[superstate]
        fn baz(event: &Event) -> Outcome<State, Response> {
            match event {
                Event::TestSuperAck => Handled(Response::ResponseOk),
                Event::TestSuperNak => Handled(Response::ResponseNak),
                _ => Handled(Response::ResponseOk),
            }
        }

        #[state]
        fn qux(event: &Event) -> Outcome<State, Response> {
            Handled(Response::ResponseOk)
        }

    }

    #[test]
    fn transition_test() {
        let mut foo = Foo.state_machine();
        
        let response = foo.handle(&Event::Transition);
        assert_eq!(response, Response::ResponseOk);   
    }

    #[test]
    fn super_ack_test() {
        let mut foo = Foo.state_machine();
        
        let response = foo.handle(&Event::TestSuperAck);
        assert_eq!(response, Response::ResponseOk);   
    }

    #[test]
    fn super_nak_test() {
        let mut foo = Foo.state_machine();

        let response = foo.handle(&Event::TestSuperNak);
        assert_eq!(response, Response::ResponseNak);
    }
}
