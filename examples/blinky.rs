use stateful::{
    state_machine,
    Response::{Handled, Super, Transition},
    Stateful,
};

// The response that will be returned by the state handlers.
type Response = stateful::Response<Blinky>;

// Define your event type.
pub enum Event {
    TimerElapsed,
    ButtonPressed,
}

// Define your data type.
pub struct Blinky {
    // The state field stores the state, which is enum derived by the
    // `state_machine` macro.
    state: State,

    // Your fields.
    light: bool,
}

// Implement the `Stateful` trait on your data type.
impl stateful::Stateful for Blinky {
    // The state enum.
    type State = State;

    // The initial state of the state machine
    const INIT_STATE: State = State::On;

    // Get a mutable reference to the current state field.
    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

// The `state_machine` macro derives an enum with variants for every state.
// Each method in the associated impl block that matches the function
// signarture of `fn(&mut Self, &Event) -> Response` is seen as a state.
#[state_machine]
// Optionally give a custom name to the state enum, the default is `State`.
#[state(name = "State")]
#[superstate(name = "Superstate")]
impl Blinky {
    // The state `On` has `Playing` as a superstate. Every time we enter
    // this state we want to call the method `enter_on`.
    #[state(name = "On", superstate = "Playing", entry_action = "Blinky::enter_on")]
    fn on(&mut self, event: &Event) -> Response {
        match event {
            // When the event `TimerElapsed` is received, transition to
            // state `Off`.
            Event::TimerElapsed => Transition(State::Off),
            _ => Super,
        }
    }

    // If no `name` field is present the name of the state enum variant will
    // be the PascalCase version of the state handler's snake_case name.
    #[state(superstate = "Playing", entry_action = "Blinky::enter_off")]
    fn off(&mut self, event: &Event) -> Response {
        match event {
            Event::TimerElapsed => Transition(State::On),
            _ => Super,
        }
    }

    #[superstate(name = "Playing")]
    fn playing(&mut self, event: &Event) -> Response {
        match event {
            Event::ButtonPressed => Transition(State::Paused),
            _ => Handled,
        }
    }

    #[state(exit_action = "Blinky::enter_paused")]
    fn paused(&mut self, event: &Event) -> Response {
        match event {
            Event::ButtonPressed => Transition(State::On),
            _ => Handled,
        }
    }
}

// Your methods.
impl Blinky {
    fn enter_on(&mut self) {
        self.light = true;
        println!("On");
    }

    fn enter_off(&mut self) {
        self.light = false;
        println!("Off");
    }

    fn enter_paused(&mut self) {
        println!("Paused");
    }
}

fn main() {
    let mut blinky = Blinky {
        state: Blinky::INIT_STATE,
        light: false,
    };

    // Calling `init()` performs the initial transition into the initial state.
    blinky.init();
    for _ in 0..10 {
        // Dispatch an event to the state machine.
        blinky.handle(&Event::TimerElapsed);
    }

    blinky.handle(&Event::ButtonPressed);

    for _ in 0..10 {
        // The state machine is paused, so the timer elapsed event does
        // not cause any transition.
        blinky.handle(&Event::TimerElapsed);
    }

    blinky.handle(&Event::ButtonPressed);

    for _ in 0..10 {
        blinky.handle(&Event::TimerElapsed);
    }
}
