use stateful::{state_machine, Stateful};

// Type alias for the generic response.
type Response = stateful::Response<Blinky>;

// The events the state machine will handle.
pub enum Event {
    TimerElapsed,
}

// The data structure on which the state machine will operate.
pub struct Blinky {
    // Your data structure must contain a `state` field to store the
    // current state.
    state: State,
}

// To make `Blinky` behave as a state machine we implement the
// `Stateful` trait on it.
impl Stateful for Blinky {
    type State = State;

    // The initial state of the state machine.
    const INIT_STATE: State = State::On;

    // Get a mutable reference to the `state` field.
    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

// The `state_machine` macro will derive a enum named `State` based on the
// state handlers tagged with the `#[state]` attribute. So in this case the
// `State` enum will be:
//
//      State {
//          On,
//          Off,
//      }
//
// This macro will also implement the `State` trait on the derived `State`
// type. See the docs for more information.
#[state_machine]
impl Blinky {
    // The `#[state]` attribute makes the state handler part of the state
    // machine. Every state handler must also have the same signature of
    // the form `fn(&mut Self, &<Self::State as State>::Event) ->
    // Response<Self>`.
    #[state]
    fn on(&mut self, event: &Event) -> Response {
        match event {
            Event::TimerElapsed => {
                println!("Off");
                // Transition to the state `Off`.
                Response::Transition(State::Off)
            }
        }
    }

    // By default the `state_machine` macro will take the snake_case name
    // of the state handler and convert it to PascalCase for the name of
    // the enum variant.
    #[state]
    fn off(&mut self, event: &Event) -> Response {
        match event {
            Event::TimerElapsed => {
                println!("On");
                // Transition to the state `On`.
                Response::Transition(State::On)
            }
        }
    }
}

fn main() {
    let mut blinky = Blinky {
        state: Blinky::INIT_STATE,
    };

    // Perform the transition into the intitial state.
    blinky.init();

    for _ in 0..10 {
        // Submit event to the state machine.
        blinky.handle(&Event::TimerElapsed);
    }
}
