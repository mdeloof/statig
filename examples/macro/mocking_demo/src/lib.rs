#![allow(unused)]

use serde::{Deserialize, Serialize};
use statig::awaitable::{InitializedStateMachine, UninitializedStateMachine};
use statig::prelude::*;

mod mockable {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default)]
    pub(crate) struct Mockable;

    #[cfg_attr(test, mockall::automock)]
    impl Mockable {
        pub(crate) fn should_transition(&self) -> bool {
            true
        }
    }
}

#[mockall_double::double]
use mockable::Mockable;

fn mockable_default() -> Mockable {
    // Allow default for `MockMockable`
    #[allow(clippy::default_constructed_unit_structs)]
    Mockable::default()
}

#[derive(Default, Deserialize, Serialize)]
pub struct Machine {
    #[serde(skip, default = "mockable_default")]
    want_to_mock: Mockable,
}

#[derive(Default, Deserialize, Serialize)]
pub struct Event;

#[state_machine(
    initial = "State::polling()",
    state(derive(Debug, Serialize, Deserialize, PartialEq, Eq))
)]
impl Machine {
    #[state]
    async fn polling(&mut self, event: &Event) -> Response<State> {
        Transition(State::downloading())
    }

    #[state]
    async fn downloading(&mut self, event: &Event) -> Response<State> {
        if self.want_to_mock.should_transition() {
            Transition(State::installing())
        } else {
            Handled
        }
    }

    #[state]
    async fn installing(&mut self, event: &Event) -> Response<State> {
        Transition(State::polling())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::fixture;
    use rstest::rstest;
    use serde_json::json;

    async fn set_init_state(
        new_state: String,
        default_machine: Machine,
    ) -> InitializedStateMachine<Machine> {
        // Create the [StateMachine<T>]
        let machine = default_machine.state_machine();

        // Serialize to a Value
        let mut intermediate_json = serde_json::to_value(&machine).unwrap();

        // Set the state
        *intermediate_json.get_mut("state").unwrap() = json!({ new_state: {} });

        // Deserialize the value
        let machine: UninitializedStateMachine<Machine> =
            serde_json::from_value(intermediate_json).unwrap();

        // Initialize the StateMachine
        machine.init().await
    }

    #[fixture]
    pub(crate) async fn downloading_fixture() -> InitializedStateMachine<Machine> {
        let machine = Machine::default();
        set_init_state("Downloading".to_string(), machine).await
    }

    #[rstest]
    #[tokio::test]
    async fn test_init_state(#[future] downloading_fixture: InitializedStateMachine<Machine>) {
        assert_eq!(*downloading_fixture.await.state(), State::downloading());
    }

    #[rstest]
    #[case(false, State::downloading())]
    #[case(true, State::installing())]
    #[tokio::test]
    async fn test_transition_to_installing(
        #[future] downloading_fixture: InitializedStateMachine<Machine>,
        #[case] transition_return: bool,
        #[case] expected_state: State,
    ) {
        // Here is the problem, in order to set expectations on the mock I need
        // to mutate inner. This can't be done when creating the [Machine]
        // because I can't the mock through serializing which is necessary for
        // me to set the initial state of the machine.
        let mut machine = downloading_fixture.await;

        machine
            .want_to_mock
            .expect_should_transition()
            .return_const(transition_return);

        machine.handle(&Event).await;

        assert_eq!(*machine.state(), expected_state,);
    }
}
