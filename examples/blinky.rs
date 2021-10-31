use stateful::{
    Response::{Handled, Parent, Transition},
    Stateful,
};

// The response that will be returned by the state handlers.
type Response = stateful::Response<Blinky>;

// Define your event type.
enum Event {
    TimerElapsed,
    ButtonPressed,
}

// Define your data type.
struct Blinky {
    // The state field stores the state.
    state: State,

    // Your fields.
    light: bool,
}

// Implement the `Stateful` trait.
impl stateful::Stateful for Blinky {
    // The event that the state machine will handle.
    type Event = Event;

    // The state enum.
    type State = State;

    // The initial state of the state machine
    const INIT_STATE: State = State::On;

    // Get a mutable reference to the current state field.
    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

// Every state is a function. The `derive_state` macro derives an enum
// with variants for every state handler. The impl block with this
// attribute should only contain state handlers.
#[stateful::derive_state]
// Name the state enum.
#[state(name = "State")]
impl Blinky {
    // The state handler `on` has `Playing` as a parent state. Every
    // time we enter this state we want to call the method `enter_on`.
    #[state(parent = "Playing", on_enter = "enter_on")]
    fn on(&mut self, event: &Event) -> Response {
        match event {
            // When the event `TimerElapsed` is received, transition to
            // state `Off`.
            Event::TimerElapsed => Transition(State::Off),
            _ => Parent,
        }
    }

    #[state(parent = "Playing", on_enter = "enter_off")]
    fn off(&mut self, event: &Event) -> Response {
        match event {
            Event::TimerElapsed => Transition(State::On),
            _ => Parent,
        }
    }

    fn playing(&mut self, event: &Event) -> Response {
        match event {
            Event::ButtonPressed => Transition(State::Paused),
            _ => Handled,
        }
    }

    #[state(on_exit = "enter_paused")]
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
