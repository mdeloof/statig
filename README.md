# Stateful

A Rust library to create hierarchial state machines. Every state is
function that handles an event or defers it to its parent state.

## Hierarchial State Machine

A hierarchial state machine (HSM) is an extension of a traditional
finite state machine (FSM). In a HSM states can also be nested inside
each other.

Consider the example of a blinking light that is turned on and off when
a timer elapses but pauses when a button is pressed. With a traditional
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
`ButtonPressed` event. In this case this isn't that big of an issue,
but as your state machine grows in complexity you'll often find that
you need to handle an event the same way in multiple states.

In a hierarchial state machine we can add a parent state `Playing` that
encapsulates the states `On` and `Off`. These child states don't handle
the `ButtonPressed` event directly but defer it to their parent state `Playing`.

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
│ Paused                               <──┘ │
├───────────────────────────────────────┤   │
│                                       │   │
│[ ButtonPressed ]──────────────────────────┘
│                                       │
└───────────────────────────────────────┘ 
```

HSM's allow you to define shared behavior for multiple states and avoid
code repetition.

## Example

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
```