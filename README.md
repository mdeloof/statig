# Stateful

Ergonomic state machines for designing event-driven systems.

**Features**

- Hierachical state machines
- State-local storage
- Compatible with `#![no_std]`, no dynamic memory allocation

---

## Stateful in action

```rust
struct Blinky {
    led: bool,
}

struct Event;

impl Stateful for Blinky {
    type State = State;

    type Input = Event;

    const INIT_STATE: State = State::on();
}

#[state_machine]
impl Blinky {
    #[state]
    fn on(&mut self, input: &Event) -> Result<State> {
        self.led = false;
        Ok(Transition(State::off()))
    }

    #[state]
    fn off(&mut self, input: &Event) -> Result<State> {
        self.led = true;
        Ok(Transition(State::on()))
    }
}

fn main() {
    let mut state_machine = StateMachine::new(Blinky { led: false });

    state_machine.handle(&Event);
}
```
(See [`/examples/basic.rs`](examples/basic.rs) for the full code with comments.)

---

## Concepts

### States

States are defined by writing methods inside the `impl` block and adding the `#[state]` attribute to them. By default the `input` argument will map to the input handled by the state machine.

```rust
#[state]
fn on(input: &Event) -> Result<State> {
    Ok(Transition(State::off()))
}
```

Every state must return a `Response` wrapped inside a `Result`. A `Response` can be one of three things: `Handled`, `Transition` or `Super`.


### Superstates

Superstates allow you to create a hierarchy of states. States can defer an input to their superstate by returning the `Super` response.

```rust
#[state(superstate = "playing")]
fn on(input: &Event) -> Result<State> {
    match input {
        Event::TimerElapsed => Ok(Transition(State::off())),
        Event::ButtonPressed => Ok(Super)
    }
}

#[superstate]
fn playing(input: &Event) -> Result<State> {
    match input {
        Event::ButtonPressed => Ok(Transition(State::paused())),
        _ => Ok(Handled)
    }
}
```

Superstates can themselves also have superstates.

### Actions

Actions run when entering or leaving states during a transition.

```rust
#[state(entry_action = "enter_on", exit_action = "exit_on")]
fn on(input: &Event) -> Result<State> {
    Ok(Transition(State::off()))
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
fn on(&mut self, input: &Event) -> Result<State> {
    self.led = false;
    Ok(Transition(State::off()))
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
fn on(counter: &mut u32, input: &Event) -> Result<State> {
    Event::TimerElapsed => {
        *counter -= 1;
        if counter  == 0 {
            Ok(Transition(State::off()))
        } else {
            Ok(Handled)
        }
    }
    Event::ButtonPressed => Ok(Super)
}
```

`counter` is only available in the `on` state but can also be accessed its superstates and actions.

### Error handling

To handle results inside your state handlers, you can use the `ResultExt` trait to map them to responses.

```rust
#[state]
fn file_open(file: &mut File, input: &Event) -> Result<State> {
    match input {
        Event::WriteRequest { data } => {
            file.write_all(data).or_transition(State::file_closed())?;
            Ok(Handled)
        }
        _ => Ok(Super)
    }
}
```


## FAQ

### **What is this `#[state_machine]` proc-macro doing to my code? 🤨**

Short answer: nothing. `#[state_machine]` simply parses the underlying `impl` block and derives some code based on its content and adds it to your source file. Your code code will still be there, unchanged. In fact `#[state_machine]` could have been a derive macro, but at the moment Rust only allows derive macros to be used on enums and structs. If you'd like to see what the generated code looks like take a look at [`tests/transition.rs`](./tests/transition.rs) and compare it with [`test/transition_macro.rs`](./tests/transition_macro.rs).

## Credits

The idea for this library came from reading the book [Practical UML Statecharts in C/C++](https://www.state-machine.com/doc/PSiCC2.pdf) and I highly recommend it if you want to learn how to use state machines to design complex systems.
