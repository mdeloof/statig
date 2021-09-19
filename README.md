# Stateful

A Rust library to create hierarchial state machines. Every state is function that handles an event or defers it to its parent state.

## Hierarchial State Machine

A hierarchial state machine (HSM) is an extension of a traditional finite state machine (FSM) where states can be nested inside each other.

Consider the example of a blinking light that is turned on and off when a timer elapses but pauses when a button is pressed. With a traditional FSM this would look something like this:

```
┌───────────────────────────────┐        
│ On                            <───┐───┐
├───────────────────────────────┤   │   │
│                               │   │   │
│[ TimerElapsed ]─────────────────┐ │   │
│                               │ │ │   │
│[ ButtonPressed ]────────────────────┐ │
│                               │ │ │ │ │
└───────────────────────────────┘ │ │ │ │
┌───────────────────────────────┐ │ │ │ │
│ Off                           <─┘ │ │ │
├───────────────────────────────┤   │ │ │
│                               │   │ │ │
│[ TimerElapsed ]───────────────────┘ │ │
│                               │     │ │
│[ ButtonPressed ]──────────────────┐ │ │
│                               │   │ │ │
└───────────────────────────────┘   │ │ │
┌───────────────────────────────┐   │ │ │
│ Paused                        <───┘─┘ │
├───────────────────────────────┤       │
│                               │       │
│[ ButtonPressed ]──────────────────────┘
│                               │        
└───────────────────────────────┘        
```

In a traditional FSM we have 3 states that all have to handle the `ButtonPressed` event. In this case this isn't that big of an issue, but as your state machine grows in complexity you'll often find that you handle an event the same way in multiple states.

In a hierarchial state machine we can add a parent state `Playing` that encapsulates the states `On` and `Off`. These child states don't handle the `ButtonPressed` event directly but defer it to their parent state `Playing`.

```
┌───────────────────────────────────────┐    
│ Playing                               <───┐
├───────────────────────────────────────┤   │
│                                       │   │
│[ ButtonPressed ]────────────────────────┐ │
│                                       │ │ │
│ ┌───────────────────────────────┐     │ │ │
│ │ On                            <───┐ │ │ │
│ ├───────────────────────────────┤   │ │ │ │
│ │                               │   │ │ │ │
│ │[ TimerElapsed ]─────────────────┐ │ │ │ │
│ │                               │ │ │ │ │ │
│ └───────────────────────────────┘ │ │ │ │ │
│ ┌───────────────────────────────┐ │ │ │ │ │
│ │ Off                           <─┘ │ │ │ │
│ ├───────────────────────────────┤   │ │ │ │
│ │                               │   │ │ │ │
│ │[ TimerElapsed ]───────────────────┘ │ │ │
│ │                               │     │ │ │
│ └───────────────────────────────┘     │ │ │
└───────────────────────────────────────┘ │ │
┌───────────────────────────────────────┐ │ │
│ Paused                                <─┘ │
├───────────────────────────────────────┤   │
│                                       │   │
│[ TimerElapsed ]───────────────────────────┘
│                                       │    
└───────────────────────────────────────┘    
```

HSM's allow us to define shared behavior for multiple states and avoid code repetition.

## Example

The blinky state machine discussed in the previous example can be implemented like this with the `stateful` crate.

```rust
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
```