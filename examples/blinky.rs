use stateful::{Response::{Handled, Transition, Parent}, Stateful};

type State = stateful::State<Blinky, Event>;
type Response = stateful::Response<Blinky, Event>;

// Events are variants of an enum.
#[derive(Clone)]
enum Event {

    // Three variants are required:
    Nop,                // No operation (used to determine hierarchy)
    OnEntry,            // On entering the state
    OnExit,             // On exiting the state

    // Then add your own:
    TimerElapsed,
    ButtonPressed
}

// The event enum must implement the `stateful::Event` trait.
// This trait provides constructors for the three required variants.
impl stateful::Event for Event {
    fn new_nop() -> Self { Event::Nop }
    fn new_on_entry() -> Self { Event::OnEntry }
    fn new_on_exit() -> Self { Event::OnExit }
}

struct Blinky {

    // The state field stores the current state
    state: State,

    // Then add your own ...
    light: bool
}

// Implement the `Stateful` trait.
impl Stateful for Blinky {

    // The event enum the state machine will be handling.
    type Event = Event;

    // The initial state of the state machine
    const INIT_STATE: State = Self::on;

    // Get a mutable reference to the current state field.
    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

// Every state is a function
impl Blinky {

    fn on(&mut self, event: &Event) -> Response {
        match event {
            Event::OnEntry => { 
                self.light = true; 
                println!("On"); 
                Handled 
            }
            Event::TimerElapsed => { 
                Transition(Self::off) 
            }
            _ => Parent(Self::playing)
        }
    }

    fn off(&mut self, event: &Event) -> Response {
        match event {
            Event::OnEntry => { 
                self.light = false; 
                println!("Off"); 
                Handled 
            }
            Event::TimerElapsed => { 
                Transition(Self::on) 
            }
            _ => Parent(Self::playing)
        }
    }

    fn playing(&mut self, event: &Event) -> Response {
        match event {
            Event::ButtonPressed => { 
                Transition(Self::paused) 
            }
            _ => Handled
        }
    }

    fn paused(&mut self, event: &Event) -> Response {
        match event {
            Event::OnEntry => { 
                println!("Paused"); 
                Handled 
            }
            Event::ButtonPressed => { 
                Transition(Self::on) 
            }
            _ => Handled
        }
    }

}

fn main() {

    let mut blinky = Blinky {
        state: Blinky::INIT_STATE,
        light: false
    };

    // Calling `init()` performs the intial transition into the initial state.
    blinky.init();

    for _ in 0..10 {
        blinky.handle(&Event::TimerElapsed);
    }

    blinky.handle(&Event::ButtonPressed);

    for _ in 0..10 {
        blinky.handle(&Event::TimerElapsed);
    }

    blinky.handle(&Event::ButtonPressed);

    for _ in 0..10 {
        blinky.handle(&Event::TimerElapsed);
    }

}