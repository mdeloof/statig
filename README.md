# Stateful

A Rust library to create state machines.

```rust
use stateful::{state_machine, Stateful};

type Response = stateful::Response<Blinky>;

pub enum Event {
    TimerElapsed,
}

pub struct Blinky {
    state: State,
}

impl Stateful for Blinky {
    type State = State;

    const INIT_STATE: State = State::On;

    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

// The `state_machine` macro derives the `State` enum.
#[state_machine]
impl Blinky {
    #[state]
    fn on(&mut self, event: &Event) -> Response {
        match event {
            Event::TimerElapsed => {
                println!("Off");
                Response::Transition(State::Off)
            }
        }
    }

    #[state]
    fn off(&mut self, event: &Event) -> Response {
        match event {
            Event::TimerElapsed => {
                println!("On");
                Response::Transition(State::On)
            }
        }
    }
}

```

(See [`/examples/basic.rs`](examples/basic.rs) for the full code with comments.)

## What is a state machine and why would I want one?

When designing a system one of the central problems you're faced with is keeping track of what has happened and what should happen going forward. State machines help you do this in a structured manner.

## Hierarchial State Machine

A hierarchial state machine (HSM) is an extension of a traditional
finite state machine (FSM). To understand why you might want to use a HSM
instead of a FSM let's look at the following example.

Consider the state machine of a blinking light that is turned on and off 
when a timer elapses but pauses when a button is pressed. With a traditional
FSM this would look something like this:

```text
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

In a traditional FSM we have 3 states that all have to handle the
`ButtonPressed` event. In this simple example that doesn't seam to be too 
much a problem, but as your state machine grows you'll often find that you
need to handle an event the same way in multiple states, resulting in code
duplication and making the state machine more and more unmanegeable as it
grows. This is what we call state explosion.

In a hierarchial state machine we can add a superstate `Playing` that
encapsulates the states `On` and `Off`. These substates don't handle
the `ButtonPressed` event directly but defer it to their superstate 
`Playing`.

```text
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
│[ ButtonPressed ]──────────────────────────┘
│                                       │
└───────────────────────────────────────┘ 
```

It doesn't have stop there. A superstate can itself be a substate of 
another superstate and so on. HSM's allow you to define shared behavior 
for multiple states and avoid code repetition. More importanly it becomes 
much easier to add new states to an existing system as the new state only 
needs to implement its unique behavior and refer to a superstate for shared
behavior.

The blinky state machine discussed in the previous example can be
implemented like this with the `stateful` crate.

```rust
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
// Name the superstate enum.
#[superstate(name = "Superstate")]
impl Blinky {

    // The state handler `on` has `Playing` as a parent state. Every
    // time we enter this state we want to call the method `enter_on`.
    #[state(parent = "Playing", entry_action = "Blinky::enter_on")]
    fn on(&mut self, event: &Event) -> Response {
        match event {
            // When the event `TimerElapsed` is received, transition to
            // state `Off`.
            Event::TimerElapsed => Transition(State::Off),
            _ => Parent,
        }
    }

    #[state(parent = "Playing", entry_action = "Blinky::enter_off")]
    fn off(&mut self, event: &Event) -> Response {
        match event {
            Event::TimerElapsed => Transition(State::On),
            _ => Parent,
        }
    }

    // The `state_machine` macro will take the snake_case name and convert
    // it to PascalCase to create the superstate variant. So `playing` becomes
    // `Playing`.
    #[superstate]
    fn playing(&mut self, event: &Event) -> Response {
        match event {
            Event::ButtonPressed => Transition(State::Paused),
            _ => Handled,
        }
    }

    #[state(entry_action = "Blinky::enter_paused")]
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
```