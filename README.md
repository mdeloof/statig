# Stateful

![CI](https://github.com/mdeloof/stateful/actions/workflows/ci.yml/badge.svg)

Hierarchial state machines for designing event-driven systems.

**Features**

- Hierachical state machines
- State-local storage
- Compatible with `#![no_std]`, no dynamic memory allocation

> **Note**
>
> At the moment `stateful` requires Rust nightly as it uses generic associated types. 
> [Stabilization for GAT's](https://github.com/rust-lang/rust/pull/96709) is on the 
> horizon however.

---

## Stateful in action

```rust
#[derive(Default)]
pub struct Blinky {
    led: bool,
}

pub enum State {
    On,
    Off,
}

impl StateMachine for Blinky {
    type State = State;

    type Superstate<'a> = ();

    type Input = Event;

    type Context = Self;

    const INIT_STATE: State = State::Off;
}

impl stateful::State<Blinky> for State {
    fn call_handler(&mut self, blinky: &mut Blinky, event: &Event) -> Response<Self> {
        match self {
            State::On => blinky.on(event),
            State::Off => blinky.off(event),
        }
    }
}

impl Blinky {
    fn on(&mut self, event: &Event) -> Response<State> {
        self.led = false;
        Transition(State::Off)
    }

    fn off(&mut self, event: &Event) -> Response<State> {
        self.led = true;
        Transition(State::On)
    }
}

fn main() {
    let mut state_machine = Blinky::state_machine().init();

    state_machine.handle(&Event);
}

```
(See the [`basic`](examples/basic/src/main.rs) example for the full code with comments.)

---

## Concepts

### States

States are defined by adding a variant to the `State` enum and writing an associated
function that can take several arguments such as the state machine input (`Event`) and
context (`Self`). The variant and function are then mapped to each other in `call_handler`.

```rust
enum State {
    On,
    Off,
}

impl Blinky {
    fn on(event: &Event) -> Response<State> {
        Transition(State::Off)
    }
}

impl stateful::State for State {
    fn call_handler(&mut self, blinky: &mut Blinky, event: &Event) -> Response<Self> {
        match self {
            State::On => blinky.on(event),
            State::Off => blinky.off(event),
        }
    }
}
```

Every state must return a `Response` which is one of three things: `Handled`, `Transition` or `Super`.

### Superstates

Superstates allow you to create a hierarchy of states. States can defer an input to their
superstate by returning the `Super` response. Superstates are defined by adding a variant 
to the `Superstate` enum and writing an associated function. The superstate is mapped to its
substates in the `superstate` method.

```rust
enum Superstate {
    Playing,
}

impl stateful::State<Blinky> for State {
    fn superstate(&mut self) -> Option<Superstate> {
        match self {
            State::On => Some(Superstate::Playing),
            State::Off => Some(Superstate::Playing),
        }
    }
}

impl stateful::Superstate<Blinky> for Superstate {
    fn call_handler(&mut self, blinky: &mut Blinky, event: &Event) -> Response<State> {
        match self {
            Superstate::Playing => blinky.playing(),
        }
    }
}
```

Superstates can themselves also have superstates.

### Actions

Actions run when entering or leaving states during a transition.

```rust
impl Blinky {
    fn enter_off() {
        println!("entered off");
    }
}

impl stateful::State<Blinky> for State {
    fn call_entry_action(&mut self, blinky: &mut Blinky) {
        match self {
            State::On => {},
            State::Off => Blinky::enter_off(),
        }
    }
}
```

### Context

If the type on which your state machine is implemented has any fields, you can access them inside all states, superstates or actions.

```rust
struct Blinky {
    led: bool
}

impl Blinky {
    fn enter_off(&self) {
        self.led = false;
        println!("entered off");
    }
}

impl stateful::State<Blinky> for State {
    fn call_entry_action(&mut self, blinky: &mut Blinky) {
        match self {
            State::On => {},
            State::Off => blinky.enter_off(),
        }
    }
}
```

### State-local storage

Sometimes you have data that only exists in a certain state. Instead of adding this data to the context and potentially having to unwrap an `Option<T>`, you can add it as an input to your state handler.

```rust
enum State {
    Off { led: bool },
    On { led: bool, counter: isize },
}

impl Blinky {
    fn on(&mut self, led: &mut bool, counter: &mut isize, event: &Event) -> Response<State> {
        *counter -= 1;
        match counter {
            0 => Transition(State::Off { led: false }),
            _ => Super
        }
    }
}

impl stateful::State<Blinky> for State {
    fn call_handler(&mut self, blinky: &mut Blinky, event: &Event) -> Response<Self> {
        match self {
            State::Off { led } => blinky.on(led, event),
            State::On { led, counter } => blinky.on(led, counter, event),
        }
    }
}   
```

`counter` is only available in the `on` state but can also be accessed in its superstates and actions.

## Credits

The idea for this library came from reading the book [Practical UML Statecharts in C/C++](https://www.state-machine.com/doc/PSiCC2.pdf). I highly recommend it if you want to learn how to use state machines to design complex systems.
