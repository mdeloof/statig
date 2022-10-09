//! # Stateful
//!
//! Hierarchical state machines for designing event-driven systems.
//!
//! **Features**
//!
//! - [Hierachically nested states](https://en.wikipedia.org/wiki/UML_state_machine#Hierarchically_nested_states)
//! - State-local storage
//! - Compatible with `#![no_std]`, no dynamic memory allocation
//! - (Optional) macro's for reducing boilerplate.
//!
//! ## Stateful in action
//!
//! ```rust
//! # use stateful::prelude::*;
//! #[derive(Default)]
//! pub struct Blinky {
//!     led: bool,
//! }
//!
//! pub struct Event;
//!
//! impl StateMachine for Blinky {
//!     type State = State;
//!     
//!     type Superstate<'a> = ();
//!     
//!     type Event = Event;
//!     
//!     type Context = Self;
//!     
//!     const INIT_STATE: State = State::off();
//! }
//!
//! #[state_machine]
//! impl Blinky {
//!     #[state]
//!     fn on(&mut self, event: &Event) -> Response<State> {
//!         self.led = false;
//!         Transition(State::off())
//!     }
//!
//!     #[state]
//!     fn off(&mut self, event: &Event) -> Response<State> {
//!         self.led = true;
//!         Transition(State::on())
//!     }
//! }
//!
//! let mut state_machine = Blinky::default().state_machine().init();
//!
//! state_machine.handle(&Event);
//! ```
//!
//! (See the [`macro/basic`](examples/macro/basic/src/main.rs) example for the full
//! code with comments. Or see [`no_macro/basic`](examples/no_macro/basic/src/main.rs)
//! for a version without using macro's).
//!
//! ## Concepts
//!
//! ### States
//!
//! States are defined by writing methods inside the `impl` block and adding the
//! `#[state]` attribute to them. By default the `event` argument will map to the
//! event handled by the state machine.
//!
//! ```rust
//! # use stateful::prelude::*;
//! # #[derive(Default)]
//! # pub struct Blinky {
//! #     led: bool,
//! # }
//! #
//! # pub struct Event;
//! #
//! # impl StateMachine for Blinky {
//! #     type State = State;
//! #     
//! #     type Superstate<'a> = ();
//! #     
//! #     type Event = Event;
//! #     
//! #     type Context = Self;
//! #     
//! #     const INIT_STATE: State = State::off();
//! # }
//! #
//! # #[state_machine]
//! # impl Blinky {
//! #
//! #[state]
//! fn on(event: &Event) -> Response<State> {
//!     Transition(State::off())
//! }
//! #
//! #     #[state]
//! #     fn off(event: &Event) -> Response<State> {
//! #         Transition(State::on())
//! #     }
//! # }
//! ```
//!
//! Every state must return a `Response`. A `Response` can be one of three things:
//!
//! - `Handled`: The event has been handled.
//! - `Transition`: Transition to another state.
//! - `Super`: Defer the event to the next superstate.
//!
//! ### Superstates
//!
//! Superstates allow you to create a hierarchy of states. States can defer an event
//! to their superstate by returning the `Super` response.
//!
//! ```
//! # use stateful::prelude::*;
//! # #[derive(Default)]
//! # pub struct Blinky {
//! #     led: bool,
//! # }
//! #
//! # pub enum Event {
//! #     TimerElapsed,
//! #     ButtonPressed
//! # }
//! #
//! # impl StateMachine for Blinky {
//! #     type State = State;
//! #     
//! #     type Superstate<'a> = Superstate;
//! #     
//! #     type Event = Event;
//! #     
//! #     type Context = Self;
//! #     
//! #     const INIT_STATE: State = State::off();
//! # }
//! #
//! # #[state_machine]
//! # impl Blinky {
//! #
//! #[state(superstate = "playing")]
//! fn on(event: &Event) -> Response<State> {
//!     match event {
//!         Event::TimerElapsed => Transition(State::off()),
//!         Event::ButtonPressed => Super
//!     }
//! }
//! #
//! #     #[state]
//! #     fn off(&mut self, event: &Event) -> Response<State> {
//! #         self.led = true;
//! #         Transition(State::on())
//! #     }
//! #
//!
//! #[superstate]
//! fn playing(event: &Event) -> Response<State> {
//!     match event {
//!         Event::ButtonPressed => Transition(State::paused()),
//!         _ => Handled
//!     }
//! }
//! #
//! #     #[state]
//! #     fn paused(event: &Event) -> Response<State> {
//! #         match event {
//! #             Event::ButtonPressed => Transition(State::on()),
//! #             _ => Super
//! #         }
//! #     }
//! # }
//! ```
//!
//! Superstates can themselves also have superstates.
//!
//! ### Actions
//!
//! Actions run when entering or leaving states during a transition.
//!
//! ```
//! # use stateful::prelude::*;
//! # #[derive(Default)]
//! # pub struct Blinky {
//! #     led: bool,
//! # }
//! #
//! # pub enum Event {
//! #     TimerElapsed,
//! #     ButtonPressed
//! # }
//! #
//! # impl StateMachine for Blinky {
//! #     type State = State;
//! #     
//! #     type Superstate<'a> = Superstate;
//! #     
//! #     type Event = Event;
//! #     
//! #     type Context = Self;
//! #     
//! #     const INIT_STATE: State = State::off();
//! # }
//! #
//! # #[state_machine]
//! # impl Blinky {
//! #     #[state]
//! #     fn off(&mut self, event: &Event) -> Response<State> {
//! #         self.led = true;
//! #         Transition(State::on())
//! #     }
//! #
//! #[state(entry_action = "enter_on", exit_action = "exit_on")]
//! fn on(event: &Event) -> Response<State> {
//!     Transition(State::off())
//! }
//!
//! #[action]
//! fn enter_on() {
//!     println!("Entered on");
//! }
//!
//! #[action]
//! fn exit_on() {
//!     println!("Exited on");
//! }
//! # }
//! ```
//!
//! ### Context
//!
//! If the type on which your state machine is implemented has any fields, you
//! can access them inside all states, superstates or actions.
//!
//! ```
//! # use stateful::prelude::*;
//! # #[derive(Default)]
//! # pub struct Blinky {
//! #     led: bool,
//! # }
//! #
//! # pub struct Event;
//! #
//! # impl StateMachine for Blinky {
//! #     type State = State;
//! #     
//! #     type Superstate<'a> = ();
//! #     
//! #     type Event = Event;
//! #     
//! #     type Context = Self;
//! #     
//! #     const INIT_STATE: State = State::off();
//! # }
//! #
//! # #[state_machine]
//! # impl Blinky {
//! #
//! #[state]
//! fn on(&mut self, event: &Event) -> Response<State> {
//!     self.led = false;
//!     Transition(State::off())
//! }
//! #
//! #     #[state]
//! #     fn off(event: &Event) -> Response<State> {
//! #         Transition(State::on())
//! #     }
//! # }
//! ```
//!
//! Or alternatively, set `led` inside the entry action.
//!
//! ```
//! # use stateful::prelude::*;
//! # #[derive(Default)]
//! # pub struct Blinky {
//! #     led: bool,
//! # }
//! #
//! # pub struct Event;
//! #
//! # impl StateMachine for Blinky {
//! #     type State = State;
//! #     
//! #     type Superstate<'a> = ();
//! #     
//! #     type Event = Event;
//! #     
//! #     type Context = Self;
//! #     
//! #     const INIT_STATE: State = State::off();
//! # }
//! #
//! # #[state_machine]
//! # impl Blinky {
//! #     #[state]
//! #     fn on(&mut self, event: &Event) -> Response<State> {
//! #         Transition(State::off())
//! #     }
//! #
//! #     #[state]
//! #     fn off(event: &Event) -> Response<State> {
//! #         Transition(State::on())
//! #     }
//! #
//! #[action]
//! fn enter_off(&mut self) {
//!     self.led = false;
//! }
//! # }
//! ```
//!
//! ### State-local storage
//!
//! Sometimes you have data that only exists in a certain state. Instead of
//! adding this data to the context and potentially having to unwrap an
//! `Option<T>`, you can add it as an input to your state handler.
//!
//! ```
//! # use stateful::prelude::*;
//! # #[derive(Default)]
//! # pub struct Blinky {
//! #     led: bool,
//! # }
//! #
//! # pub enum Event {
//! #     TimerElapsed,
//! #     ButtonPressed
//! # }
//! #
//! # impl StateMachine for Blinky {
//! #     type State = State;
//! #     
//! #     type Superstate<'a> = ();
//! #     
//! #     type Event = Event;
//! #     
//! #     type Context = Self;
//! #     
//! #     const INIT_STATE: State = State::on(10);
//! # }
//! #
//! # #[state_machine]
//! # impl Blinky {
//! #
//! #[state]
//! fn on(counter: &mut u32, event: &Event) -> Response<State> {
//!     match event {
//!         Event::TimerElapsed => {
//!             *counter -= 1;
//!             if *counter == 0 {
//!                 Transition(State::off())
//!             } else {
//!                 Handled
//!             }
//!         }
//!         Event::ButtonPressed => Transition(State::on(10))
//!     }
//! }
//! #
//! #     #[state]
//! #     fn off(event: &Event) -> Response<State> {
//! #         Transition(State::on(10))
//! #     }
//! # }
//! ```
//!
//! `counter` is only available in the `on` state but can also be accessed in
//! its superstates and actions.
//!
//! ## FAQ
//!
//! ### **What is this `#[state_machine]` proc-macro doing to my code? 🤨**
//!
//! Short answer: nothing. `#[state_machine]` simply parses the underlying `impl`
//! block and derives some code based on its content and adds it to your source
//! file. Your code will still be there, unchanged. In fact `#[state_machine]`
//! could have been a derive macro, but at the moment Rust only allows derive macros
//! to be used on enums and structs. If you'd like to see what the generated code
//! looks like take a look at the test [with](./stateful/tests/transition_macro.rs)
//! and [without](./stateful/tests/transition.rs) macros.
//!
//! ## Credits
//!
//! The idea for this library came from reading the book
//! [Practical UML Statecharts in C/C++](https://www.state-machine.com/doc/PSiCC2.pdf).
//! I highly recommend it if you want to learn how to use state machines to design
//! complex systems.

#![no_std]

mod state;
mod state_machine;
mod superstate;

pub use state::*;
pub use state_machine::*;
pub use superstate::*;

/// Macro for deriving the state and superstate enum.
///
/// By parsing the underlying `impl` block and searching for methods with the
/// `state`, `superstate` or `action` attribute, the `state_machine` macro can
/// derive the state and superstate enums. By default these will be given the
/// names '`State`' and '`Superstate`'. Next to that the macro will also
/// implement the [`State`](crate::State) trait for the state enum and the
/// [`Superstate`](crate::Superstate) trait for the superstate enum.
///
/// To override the default configuration you can use the following attributes.
///
/// - `#[state_machine(state(name = "CustomStateName"))]`
///
///   Set the name of the state enum to a custom name.
///
///   _Default_: `State`
///
///   </br>
///
/// - `#[state_machine(superstate(name = "CustomSuperstateName"))]`
///
///   Set the name of the superstate enum to a custom name.
///
///   _Default_: `Superstate`
///   
///   </br>
///
/// - `#[state_machine(state(derive(SomeTrait, AnotherTrait)))]`
///
///   Apply the derive macro with the passed traits to the state enum.
///
///   _Default_: `()`
///
///   </br>
///
/// - `#[state_machine(superstate(derive(SomeTrait, AnotherTrait)))]`
///
///   Apply the derive macro with the passed traits to the superstate enum.
///
///   _Default_: `()`
///
///   </br>
#[cfg(feature = "macro")]
pub use stateful_macro::state_machine;

/// Attribute for tagging a state.
///
/// This macro does nothing on its own but is detected by the `state_machine`
/// macro when added to a method.
///
/// It accepts the following attributes:
///
/// - `#[state(name = "CustomStateName")]`
///
///   Set the name of the variant that will be part of the state enum.
///
///   </br>
///
/// - `#[state(superstate = "superstate_name")]`
///
///   Set the superstate of the state.
///
///   </br>
///
/// - `#[state(entry_action = "entry_action_name")]`
///
///   Set the entry action of the state.
///
///   </br>
///
/// - `#[state(exit_action = "exit_action_name")]`
///
///   Set the exit action of the state.
///
///   </br>
///
/// - `#[state(local_storage("field_name_a: FieldTypeA", "field_name_b: FieldTypeB"))]`
///
///   Add local storage to this state. These will be added as fields to the enum variant.
///
///   </br>
#[cfg(feature = "macro")]
pub use stateful_macro::state;

/// Attribute for tagging a superstate.
///
/// This macro does nothing on its own but is detected by the `state_machine`
/// macro when added to a method.
///
/// It accepts the following attributes:
///
/// - `#[superstate(name = "CustomSuperstateName")]`
///
///   Set the name of the variant that will be part of the state enum.
///
///   </br>
///
/// - `#[superstate(superstate = "superstate_name")]`
///
///   Set the superstate of the superstate.
///
///   </br>
///
/// - `#[superstate(entry_action = "entry_action_name")]`
///
///   Set the entry action of the superstate.
///
///   </br>
///
/// - `#[superstate(exit_action = "exit_action_name")]`
///
///   Set the exit action of the superstate.
///
///   </br>
///
/// - `#[superstate(local_storage("field_name_a: &'a mut FieldTypeA"))]`
///
///   Add local storage to this superstate. These will be added as fields to
///   the enum variant. It is crucial to understand that superstates never own
///   their data. Instead it is always borrowed from the underlying state or
///   superstate. This means the fields should be references with an
///   assoaciated lifetime `'a`.
///
///   </br>
#[cfg(feature = "macro")]
pub use stateful_macro::superstate;

/// Attribute for tagging an action.
///
/// This macro does nothing on its own but is detected by the `state_machine`
/// macro when added to a method.
#[cfg(feature = "macro")]
pub use stateful_macro::action;

pub mod prelude {
    pub use crate::Response::{self, *};
    pub use crate::{
        State, StateExt, StateMachine, StateMachineContextEqSelfExt, StateMachineContextNeSelfExt,
        Superstate, SuperstateExt,
    };

    #[cfg(feature = "macro")]
    pub use stateful_macro::state_machine;
}

/// Response that can be returned by a state machine.
pub enum Response<S> {
    /// Consider the event handled.
    Handled,
    /// Defer the event to the superstate.
    Super,
    /// Transition to the given state.
    Transition(S),
}
