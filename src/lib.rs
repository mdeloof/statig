//!
//! # Stateful
//!
//! A Rust library to create hierarchial state machines. Every state is
//! function that handles an event or defers it to its parent state.
//!
//! ## Hierarchial State Machine
//!
//! A hierarchial state machine (HSM) is an extension of a traditional
//! finite state machine (FSM). In a HSM states can also be nested inside
//! each other.
//!
//! Consider the example of a blinking light that is turned on and off when
//! a timer elapses but pauses when a button is pressed. With a traditional
//! FSM this would look something like this:
//!
//! ```text
//! ┌───────────────────────────────┐        
//! │ On                            <───┐───┐
//! ├───────────────────────────────┤   │   │
//! │                               │   │   │
//! │[ TimerElapsed ]─────────────────┐ │   │
//! │                               │ │ │   │
//! │[ ButtonPressed ]────────────────────┐ │
//! │                               │ │ │ │ │
//! └───────────────────────────────┘ │ │ │ │
//! ┌───────────────────────────────┐ │ │ │ │
//! │ Off                           <─┘ │ │ │
//! ├───────────────────────────────┤   │ │ │
//! │                               │   │ │ │
//! │[ TimerElapsed ]───────────────────┘ │ │
//! │                               │     │ │
//! │[ ButtonPressed ]──────────────────┐ │ │
//! │                               │   │ │ │
//! └───────────────────────────────┘   │ │ │
//! ┌───────────────────────────────┐   │ │ │
//! │ Paused                        <───┘─┘ │
//! ├───────────────────────────────┤       │
//! │                               │       │
//! │[ ButtonPressed ]──────────────────────┘
//! │                               │        
//! └───────────────────────────────┘        
//! ```
//!
//! In a traditional FSM we have 3 states that all have to handle the
//! `ButtonPressed` event. In this case this isn't that big of an issue,
//! but as your state machine grows in complexity you'll often find that
//! you need to handle an event the same way in multiple states.
//!
//! In a hierarchial state machine we can add a parent state `Playing` that
//! encapsulates the states `On` and `Off`. These child states don't handle
//! the `ButtonPressed` event directly but defer it to their parent state `Playing`.
//!
//! ```text
//! ┌───────────────────────────────────────┐    
//! │ Playing                               <───┐
//! ├───────────────────────────────────────┤   │
//! │                                       │   │
//! │[ ButtonPressed ]────────────────────────┐ │
//! │                                       │ │ │
//! │ ┌───────────────────────────────┐     │ │ │
//! │ │ On                            <───┐ │ │ │
//! │ ├───────────────────────────────┤   │ │ │ │
//! │ │                               │   │ │ │ │
//! │ │[ TimerElapsed ]─────────────────┐ │ │ │ │
//! │ │                               │ │ │ │ │ │
//! │ └───────────────────────────────┘ │ │ │ │ │
//! │ ┌───────────────────────────────┐ │ │ │ │ │
//! │ │ Off                           <─┘ │ │ │ │
//! │ ├───────────────────────────────┤   │ │ │ │
//! │ │                               │   │ │ │ │
//! │ │[ TimerElapsed ]───────────────────┘ │ │ │
//! │ │                               │     │ │ │
//! │ └───────────────────────────────┘     │ │ │
//! └───────────────────────────────────────┘ │ │
//! ┌───────────────────────────────────────┐ │ │
//! │ Paused                               <──┘ │
//! ├───────────────────────────────────────┤   │
//! │                                       │   │
//! │[ ButtonPressed ]──────────────────────────┘
//! │                                       │    
//! └───────────────────────────────────────┘    
//! ```
//!
//! HSM's allow you to define shared behavior for multiple states and avoid
//! code repetition.
//!
//! ## Example
//!
//! The blinky state machine discussed in the previous example can be
//! implemented like this with the `stateful` crate.
//!
//! ```
//! use stateful::{
//!     Response::{Handled, Parent, Transition},
//!     Stateful,
//! };
//!
//! // The response that will be returned by the state handlers.
//! type Response = stateful::Response<Blinky>;
//!
//! // Define your event type.
//! enum Event {
//!     TimerElapsed,
//!     ButtonPressed,
//! }
//!
//! // Define your data type.
//! struct Blinky {
//!     // The state field stores the state.
//!     state: State,
//!     
//!     // Your fields.
//!     light: bool,
//! }
//!
//! // Implement the `Stateful` trait.
//! impl stateful::Stateful for Blinky {
//!     // The event that the state machine will handle.
//!     type Event = Event;
//!     
//!     // The state enum.
//!     type State = State;
//!
//!     // The initial state of the state machine
//!     const INIT_STATE: State = State::On;
//!
//!     // Get a mutable reference to the current state field.
//!     fn state_mut(&mut self) -> &mut State {
//!         &mut self.state
//!     }
//! }
//!
//! // Every state is a function. The `derive_state` macro derives an enum
//! // with variants for every state handler. The impl block with this
//! // attribute should only contain state handlers.
//! #[stateful::derive_state]
//! // Name the state enum.
//! #[state(name = "State")]
//! impl Blinky {
//!     
//!     // The state handler `on` has `Playing` as a parent state. Every
//!     // time we enter this state we want to call the method `enter_on`.
//!     #[state(parent = "Playing", on_enter = "Blinky::enter_on")]
//!     fn on(&mut self, event: &Event) -> Response {
//!         match event {
//!             // When the event `TimerElapsed` is received, transition to
//!             // state `Off`.
//!             Event::TimerElapsed => Transition(State::Off),
//!             _ => Parent,
//!         }
//!     }
//!
//!     #[state(parent = "Playing", on_enter = "Blinky::enter_off")]
//!     fn off(&mut self, event: &Event) -> Response {
//!         match event {
//!             Event::TimerElapsed => Transition(State::On),
//!             _ => Parent,
//!         }
//!     }
//!
//!     fn playing(&mut self, event: &Event) -> Response {
//!         match event {
//!             Event::ButtonPressed => Transition(State::Paused),
//!             _ => Handled,
//!         }
//!     }
//!
//!     #[state(on_exit = "Blinky::enter_paused")]
//!     fn paused(&mut self, event: &Event) -> Response {
//!         match event {
//!             Event::ButtonPressed => Transition(State::On),
//!             _ => Handled,
//!         }
//!     }
//! }
//!
//! // Your methods.
//! impl Blinky {
//!     fn enter_on(&mut self) {
//!         self.light = true;
//!         println!("On");
//!     }
//!
//!     fn enter_off(&mut self) {
//!         self.light = false;
//!         println!("Off");
//!     }
//!
//!     fn enter_paused(&mut self) {
//!         println!("Paused");
//!     }
//! }
//!
//! fn main() {
//!     let mut blinky = Blinky {
//!         state: Blinky::INIT_STATE,
//!         light: false,
//!     };
//!
//!     // Calling `init()` performs the initial transition into the initial state.
//!     blinky.init();
//!
//!     for _ in 0..10 {
//!         // Dispatch an event to the state machine.
//!         blinky.handle(&Event::TimerElapsed);
//!     }
//!
//!     blinky.handle(&Event::ButtonPressed);
//!
//!     for _ in 0..10 {
//!         // The state machine is paused, so the timer elapsed event does
//!         // not cause any transition.
//!         blinky.handle(&Event::TimerElapsed);
//!     }
//!
//!     blinky.handle(&Event::ButtonPressed);
//!
//!     for _ in 0..10 {
//!         blinky.handle(&Event::TimerElapsed);
//!     }
//! }
//! ```

#![no_std]

pub mod vec;

use crate::vec::Vec;

pub use stateful_derive::*;

/// Type alias for the state function pointer.
pub type StateHandler<T, E> = fn(&mut T, &E) -> Response<T>;

/// Type alias for the state on enter handler.
pub type StateOnEnterHandler<T> = fn(&mut T);

/// Type alias for the state on exit handler.
pub type StateOnExitHandler<T> = fn(&mut T);

/// The maximum depth states can be nested inside each other.
const DEPTH: usize = 16;

/// The response returned by a state handler function.
pub enum Response<T: Stateful> {
    /// The event has been handled.
    Handled,
    /// Defer the event to the parent state.
    Parent,
    /// Transition to a leaf state.
    Transition(T::State),
}

/// Trait that should be implemented on your struct.
pub trait Stateful: Sized {
    /// The event that will be handled by the state machine.
    type Event;

    /// The state enum that
    type State: State<Object = Self, Event = Self::Event>;

    /// The initial state of the state machine.
    const INIT_STATE: Self::State;

    /// Get a mutable reference to the current state.
    fn state_mut(&mut self) -> &mut Self::State;

    /// Perform the transition into the initial state starting from the
    /// root state.
    fn init(&mut self) {
        let init_state = *self.state_mut();
        self.drill_into(init_state);
    }

    /// Perform the transition out of the current state and reset the
    /// init state.
    fn deinit(&mut self) {
        let state = *self.state_mut();
        self.drill_out_of(state);
        *self.state_mut() = Self::INIT_STATE;
    }

    /// Transition from the outside into the given state.
    fn drill_into(&mut self, state: Self::State) {
        let entry_path = state.parent_path();
        for state in entry_path.into_iter().rev() {
            if let Some(state_enter_handler) = state.state_on_enter_handler() {
                state_enter_handler(self);
            }
        }
    }

    /// Transition from the inner state to the outside.
    fn drill_out_of(&mut self, state: Self::State) {
        let exit_path = state.parent_path();
        for state in exit_path.into_iter() {
            if let Some(state_enter_handler) = state.state_on_exit_handler() {
                state_enter_handler(self);
            }
        }
    }

    /// Handle an event from within the current state.
    fn handle(&mut self, event: &Self::Event) {
        let state = *self.state_mut();
        self.call_handler(state, event);
    }

    /// Handle an event from a given state.
    fn call_handler(&mut self, state: Self::State, event: &Self::Event) {
        match (state.state_handler())(self, event) {
            Response::Transition(target_state) => self.transition(target_state),
            Response::Parent => match state.parent_state() {
                Some(parent) => self.call_handler(parent, event),
                None => (),
            },
            Response::Handled => (),
        }
    }

    /// Perform a transition from the current state towards the target
    /// state.
    fn transition(&mut self, target: Self::State)
    where
        Self: Sized,
    {
        let mut exit_path: Vec<Self::State, DEPTH> = Vec::new();
        let mut entry_path: Vec<Self::State, DEPTH> = Vec::new();
        let source = *self.state_mut();

        let mut exit_temp = source;
        let mut entry_temp = target;

        // Get the path from the source state to the root state
        for i in 0..DEPTH {
            exit_path.push(exit_temp);
            match exit_temp.parent_state() {
                Some(parent_state) => exit_temp = parent_state,
                // Reached the top state
                None => break,
            }
            assert_ne!(i, DEPTH, "Reached max state nesting depth of {}", DEPTH);
        }

        // Get the path from the target state to the root states
        for i in 0..DEPTH {
            entry_path.push(entry_temp);
            match entry_temp.parent_state() {
                Some(parent_state) => entry_temp = parent_state,
                // Reached the top state
                None => break,
            }
            assert_ne!(i, DEPTH, "Reached max state nesting depth of {}", DEPTH);
        }

        // Starting from the root state, trim the entry and exit paths so
        // only uncommon states remain.
        for i in 0..DEPTH {
            // If all states are descendants of a single root state, there
            // will always be at leat one shared shared parent state in the
            // entry and exit paths.
            entry_temp = *entry_path.last().expect(
                "Only perform transitions to leaf states, i.e. states
                 that do not contain other sub-states",
            );
            exit_temp = *exit_path.last().expect(
                "Only perform transitions to leaf states, i.e. states
                 that do not contain other sub-states",
            );
            if exit_temp != entry_temp {
                // Found the top most parent state that is not shared
                break;
            } else {
                // The parent state is shared, so we should remove it from
                // the path. But if this is also the last state in both
                // paths that means we're dealing with a self-transition.
                // In that case we keep this state in the entry and exit
                // paths, and break out of the loop.
                if entry_path.len() == 1 && exit_path.len() == 1 {
                    break;
                } else {
                    entry_path.pop();
                    exit_path.pop();
                }
            }
            assert_ne!(i, DEPTH, "Reached max state nesting depth of {}", DEPTH);
        }

        // Execute the exit path out of the source state
        for state in exit_path.into_iter() {
            if let Some(state_exit_handler) = state.state_on_exit_handler() {
                state_exit_handler(self);
            }
        }

        // Execute the entry path into the target state
        for state in entry_path.into_iter().rev() {
            if let Some(state_exit_handler) = state.state_on_enter_handler() {
                state_exit_handler(self);
            }
        }
        *self.state_mut() = target;
    }
}

/// Trait that should be implemented on the state enum.
pub trait State: Sized + Copy + PartialEq {
    /// The object on which the state handlers operate.
    type Object: Stateful;

    /// The event that is handled by the state handlers.
    type Event;

    /// Get the associated state handler.
    fn state_handler(&self) -> StateHandler<Self::Object, Self::Event>;

    /// Get the associated parent state, if defined.
    fn parent_state(&self) -> Option<Self>;

    /// Get the associated `on_enter` handler, if defined.
    fn state_on_enter_handler(&self) -> Option<StateOnEnterHandler<Self::Object>>;

    /// Get the associated `on_exit` handler, if definded.
    fn state_on_exit_handler(&self) -> Option<StateOnExitHandler<Self::Object>>;

    /// Get the path towards the root from a given state.
    fn parent_path(&self) -> Vec<Self, DEPTH> {
        let mut path: Vec<Self, DEPTH> = Vec::new();
        let mut exit_temp = *self;
        for i in 0..DEPTH {
            path.push(exit_temp);
            match exit_temp.parent_state() {
                Some(parent_state) => exit_temp = parent_state,
                // Reached the top state
                None => break,
            }
            if i == DEPTH - 1 {
                panic!("reached max state nesting depth of {}", DEPTH)
            }
        }
        path
    }
}
