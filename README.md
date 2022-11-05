# statig

![CI](https://github.com/mdeloof/statig/actions/workflows/ci.yml/badge.svg)

Hierarchical state machines for designing event-driven systems.

**Features**

- Hierachical state machines
- State-local storage
- Compatible with `#![no_std]`, no dynamic memory allocation
- (Optional) macro's for reducing boilerplate.

---

## statig in action

```rust
#[derive(Default)]
pub struct Blinky {
    led: bool,
}

pub struct Event;

impl StateMachine for Blinky { 
    type State = State;
    
    type Superstate<'a> = Superstate;
    
    type Event = Event;
    
    const INIT_STATE: State = State::off();
}

#[state_machine]
impl Blinky {
    #[state]
    fn on(&mut self, event: &Event) -> Response<State> {
        self.led = false;
        Transition(State::off())
    }

    #[state]
    fn off(&mut self, event: &Event) -> Response<State> {
        self.led = true;
        Transition(State::on())
    }
}

fn main() {
    let mut state_machine = Blinky::default().state_machine().init();

    state_machine.handle(&Event);
}
```

(See the [`macro/basic`](examples/macro/basic/src/main.rs) example for the full code with comments. Or see [`no_macro/basic`](examples/no_macro/basic/src/main.rs) for a version without using macro's).


---

## Concepts

### States

States are defined by writing methods inside the `impl` block and adding the `#[state]` attribute to them. By default the `event` argument will map to the event handled by the state machine.

```rust
#[state]
fn on(event: &Event) -> Response<State> {
    Transition(State::off())
}
```

Every state must return a `Response`. A `Response` can be one of three things:

- `Handled`: The event has been handled.
- `Transition`: Transition to another state.
- `Super`: Defer the event to the next superstate.

### Superstates

Superstates allow you to create a hierarchy of states. States can defer an event to their superstate by returning the `Super` response.

```rust
#[state(superstate = "playing")]
fn on(event: &Event) -> Response<State> {
    match event {
        Event::TimerElapsed => Transition(State::off()),
        Event::ButtonPressed => Super
    }
}

#[superstate]
fn playing(event: &Event) -> Response<State> {
    match event {
        Event::ButtonPressed => Transition(State::paused()),
        _ => Handled
    }
}
```

Superstates can themselves also have superstates.

### Actions

Actions run when entering or leaving states during a transition.

```rust
#[state(entry_action = "enter_on", exit_action = "exit_on")]
fn on(event: &Event) -> Response<State> {
    Transition(State::off())
}

#[action]
fn enter_on() {
    println!("Entered on");
}

#[action]
fn exit_on() {
    println!("Exited on");
}
```

### Context

If the type on which your state machine is implemented has any fields, you can access them inside all states, superstates or actions.

```rust
#[state]
fn on(&mut self, event: &Event) -> Response<State> {
    self.led = false;
    Transition(State::off())
}
```

Or alternatively, set `led` inside the entry action.

```rust
#[action]
fn enter_off(&mut self) {
    self.led = false;
}
```

### State-local storage

Sometimes you have data that only exists in a certain state. Instead of adding this data to the context and potentially having to unwrap an `Option<T>`, you can add it as an input to your state handler.

```rust
#[state]
fn on(counter: &mut u32, event: &Event) -> Response<State> {
    match event {
        Event::TimerElapsed => {
            *counter -= 1;
            if *counter == 0 {
                Transition(State::off())
            } else {
                Handled
            }
        }
        Event::ButtonPressed => Transition(State::on(10))
    }
}
```

`counter` is only available in the `on` state but can also be accessed in its superstates and actions.

## FAQ

### **What is this `#[state_machine]` proc-macro doing to my code? 🤨**

Short answer: nothing. `#[state_machine]` simply parses the underlying `impl` block and derives some code based on its content and adds it to your source file. Your code will still be there, unchanged. In fact `#[state_machine]` could have been a derive macro, but at the moment Rust only allows derive macros to be used on enums and structs. If you'd like to see what the generated code looks like take a look at the test [with](./statig/tests/transition_macro.rs) and [without](./statig/tests/transition.rs) macros.

## Credits

The idea for this library came from reading the book [Practical UML Statecharts in C/C++](https://www.state-machine.com/doc/PSiCC2.pdf). I highly recommend it if you want to learn how to use state machines to design complex systems.
