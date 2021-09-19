//!
//! # Stateful
//! 
//! A Rust library to create hierarchial state machines. Every state is 
//! function that handles an event or defers it to its parent state.
//! 
//! ## Hierarchial State Machine
//! 
//! A hierarchial state machine (HSM) is an extension of a traditional 
//! finite state machine (FSM) where states can be nested inside each other.
//! 
//! Consider the example of a blinking light that is turned on and off when 
//1 a timer elapses but pauses when a button is pressed. With a traditional 
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
//! you handle an event the same way in multiple states.
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
//! HSM's allow us to define shared behavior for multiple states and avoid 
//! ode repetition.
//! 
//! ## Example
//! 
//! The blinky state machine discussed in the previous example can be 
//! implemented like this with the `stateful` crate.
//! 
//! ```rust
//! use stateful::{Response::{Handled, Transition, Parent}, Stateful, StateWrapper, State};
//! 
//! //type State = stateful::State<Blinky, Event>;
//! type Response = stateful::Response<Blinky, Event>;
//! 
//! // Events are variants of an enum.
//! #[derive(Clone)]
//! enum Event {
//! 
//!     // Three variants are required:
//!     Nop,                // No operation (used to determine hierarchy)
//!     OnEntry,            // On entering the state
//!     OnExit,             // On exiting the state
//! 
//!     // Then add your own:
//!     TimerElapsed,
//!     ButtonPressed
//! }
//! 
//! // The event enum must implement the `stateful::Event` trait.
//! // This trait provides constructors for the three required variants.
//! impl stateful::Event for Event {
//!     fn new_nop() -> Self { Event::Nop }
//!     fn new_on_entry() -> Self { Event::OnEntry }
//!     fn new_on_exit() -> Self { Event::OnExit }
//! }
//! 
//! struct Blinky {
//! 
//!     // The state field stores the current state
//!     state: StateWrapper<Self, Event>,
//! 
//!     // Then add your own ...
//!     light: bool
//! }
//! 
//! // Implement the `Stateful` trait.
//! impl Stateful for Blinky {
//! 
//!     // The event enum the state machine will be handling.
//!     type Event = Event;
//! 
//!     // The initial state of the state machine
//!     const INIT_STATE: State<Self, Event> = Self::on;
//! 
//!     // Get a mutable reference to the current state field.
//!     fn state_mut(&mut self) -> &mut State<Self, Event> {
//!         &mut self.state.0
//!     }
//! }
//! 
//! // Every state is a function
//! impl Blinky {
//! 
//!     fn on(&mut self, event: &Event) -> Response {
//!         match event {
//!             Event::OnEntry => { 
//!                 self.light = true; 
//!                 println!("On"); 
//!                 Handled 
//!             }
//!             Event::TimerElapsed => { 
//!                 Transition(Self::off) 
//!             }
//!             _ => Parent(Self::playing)
//!         }
//!     }
//! 
//!     fn off(&mut self, event: &Event) -> Response {
//!         match event {
//!             Event::OnEntry => { 
//!                 self.light = false; 
//!                 println!("Off"); 
//!                 Handled 
//!             }
//!             Event::TimerElapsed => { 
//!                 Transition(Self::on) 
//!             }
//!             _ => Parent(Self::playing)
//!         }
//!     }
//! 
//!     fn playing(&mut self, event: &Event) -> Response {
//!         match event {
//!             Event::ButtonPressed => { 
//!                 Transition(Self::paused) 
//!             }
//!             _ => Handled
//!         }
//!     }
//! 
//!     fn paused(&mut self, event: &Event) -> Response {
//!         match event {
//!             Event::OnEntry => { 
//!                 println!("Paused"); 
//!                 Handled 
//!             }
//!             Event::ButtonPressed => { 
//!                 Transition(Self::on) 
//!             }
//!             _ => Handled
//!         }
//!     }
//! 
//! }
//! 
//! fn main() {
//! 
//!     let mut blinky = Blinky {
//!         state: StateWrapper(Blinky::INIT_STATE),
//!         light: false
//!     };
//! 
//!     // Calling `init()` performs the intial transition into the initial state.
//!     blinky.init();
//! 
//!     for _ in 0..10 {
//!         blinky.handle(&Event::TimerElapsed);
//!     }
//! 
//!     blinky.handle(&Event::ButtonPressed);
//! 
//!     for _ in 0..10 {
//!         blinky.handle(&Event::TimerElapsed);
//!     }
//! 
//!     blinky.handle(&Event::ButtonPressed);
//! 
//!     for _ in 0..10 {
//!         blinky.handle(&Event::TimerElapsed);
//!     }
//! 
//! }
//! ```
//!

use std::fmt;

/// Type alias for the state function pointer.
pub type State<T, E> = fn(&mut T , &E) -> Response<T, E>;

/// Wrapper for the state function pointer that implements `Debug`.
pub struct StateWrapper<T, E>(pub fn(&mut T , &E) -> Response<T, E>)
where E: Event + Clone;

impl<T, E> Clone for StateWrapper<T, E>
where E: Event {

    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T, E> Copy for StateWrapper<T, E>
where E: Event {}

impl<T, E> fmt::Debug for StateWrapper<T, E>
where E: Event {
    
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("State")
         .field("state", &"cool")
         .finish()
    }
}

/// The response returned by a state handler function.
pub enum Response<T, E: Event> {
    /// The event has been handled.
    Handled,
    /// Defer the event to the parent state.
    Parent(State<T, E>),
    /// Transition to a leaf state.
    Transition(State<T, E>)
}

/// Trait that should be implemented for the event that will be handled by
/// the state machine.
pub trait Event: Clone {

    /// Constructor for nop (no-operation) event.
    fn new_nop() -> Self;

    /// Constructor for on entry event.
    fn new_on_entry() -> Self;

    /// Constructor for on exit event.
    fn new_on_exit() -> Self;

}

/// Trait that should be implemented on your struct.
pub trait Stateful: Sized {
    /// The event that will be handled by the state machine.
    type Event: Event;

    /// The initial state of the state machine.
    const INIT_STATE: State<Self, Self::Event>;

    /// The max depth the states can be nested inside each other. The
    /// default is 16.
    const MAX_DEPTH: usize = 16;

    /// Get a mutable reference to the current state.
    fn state_mut(&mut self) -> &mut State<Self, Self::Event>;

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

    fn drill_into(&mut self, state: State<Self, Self::Event>) {
        let entry_path = self.parent_path(state);
        // Execute the entry path into the target state
        let entry_event = Self::Event::new_on_entry();
        for entry_state in entry_path.into_iter().rev() {
            match (entry_state)(self, &entry_event) {
                Response::Handled => {},
                Response::Transition(_) => panic!(
                    "do not perform transition on entry event"),
                _ => {}
            }
        }
    }

    fn drill_out_of(&mut self, state: State<Self, Self::Event>) {
        let exit_path = self.parent_path(state);
        // Execute the entry path into the target state
        let entry_event = Self::Event::new_on_entry();
        for entry_state in exit_path.into_iter() {
            match (entry_state)(self, &entry_event) {
                Response::Handled => {},
                Response::Transition(_) => panic!(
                    "do not perform transition on exit event"),
                _ => {}
            }
        }
    }

    /// Handle an event from within the current state.
    fn handle(&mut self, event: &Self::Event) {
        let state = *self.state_mut();
        self.call_handler(state, event);
    }

    /// Handle an event from a given state.
    fn call_handler(&mut self, handler: State<Self, Self::Event>, event: &Self::Event) {
        match (handler)(self, event) {
            Response::Transition(target_state) => self.transition(target_state),
            Response::Parent(parent_state) => self.call_handler(parent_state, event),
            Response::Handled => ()
        }
    }

    /// Get the parent state of a given state. If a state has no parent
    /// state (most likely because it is the root state) the result will 
    /// be an error.
    fn parent_state(&mut self, state: State<Self, Self::Event>) -> Option<State<Self, Self::Event>> {
        let nop_event = Self::Event::new_nop();
        return match (state)(self, &nop_event) {
            Response::Parent(state) => Some(state),
            _ => None
        }
    }

    /// Get the path towards the root from a given state.
    fn parent_path(&mut self, state: State<Self, Self::Event>) -> Vec<State<Self, Self::Event>> {
        let mut path: Vec<State<Self, Self::Event>> = Vec::with_capacity(Self::MAX_DEPTH);
        let mut exit_temp = state;
        for i in 0..(Self::MAX_DEPTH + 1) {
            path.push(exit_temp);
            match self.parent_state(exit_temp) {
                Some(parent_state) => exit_temp = parent_state,
                // Reached the top state
                None => break
            }
            if i == Self::MAX_DEPTH {
                panic!("reached max state nesting depth of {}", Self::MAX_DEPTH)
            }
        }
        path
    }

    /// Perform a transition from the current state towards the target
    /// state.
    fn transition(&mut self, target: State<Self, Self::Event>)
    where Self: Sized {
        let mut exit_path: Vec<State<Self, Self::Event>> = Vec::with_capacity(Self::MAX_DEPTH);
        let mut entry_path: Vec<State<Self, Self::Event>> = Vec::with_capacity(Self::MAX_DEPTH);
        let source = *self.state_mut();

        let mut exit_temp = source;
        let mut entry_temp = target;

        // Get the path from the source state to the root state
        for i in 0..(Self::MAX_DEPTH + 1) {
            exit_path.push(exit_temp);
            match self.parent_state(exit_temp) {
                Some(parent_state) => exit_temp = parent_state,
                // Reached the top state
                None => break
            }
            assert_ne!(i, Self::MAX_DEPTH, "Reached max state nesting depth of {}", Self::MAX_DEPTH);
        }

        // Get the path from the target state to the root states
        for i in 0..(Self::MAX_DEPTH + 1) {
            entry_path.push(entry_temp);
            match self.parent_state(entry_temp) {
                Some(parent_state) => entry_temp = parent_state,
                // Reached the top state
                None => break
            }
            assert_ne!(i, Self::MAX_DEPTH, "Reached max state nesting depth of {}", Self::MAX_DEPTH);
        }

        // Starting from the root state, trim the entry and exit paths so
        // only uncommon states remain.
        for i in 0..(Self::MAX_DEPTH + 1) {
            // If all states are descendants of a single root state, there
            // will always be at leat one shared shared parent state in the 
            // entry and exit paths.
            entry_temp = *entry_path.last().expect(
                "Only perform transitions to leaf states, i.e. states
                 that do not contain other sub-states");
            exit_temp = *exit_path.last().expect(
                "Only perform transitions to leaf states, i.e. states
                 that do not contain other sub-states");
            if (exit_temp) as usize != (entry_temp) as usize {
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
            assert_ne!(i, Self::MAX_DEPTH, "Reached max state nesting depth of {}", Self::MAX_DEPTH);
        }

        // Execute the exit path out of the source state
        let exit_event = Self::Event::new_on_exit();
        for exit_state in exit_path.into_iter() {
            match (exit_state)(self, &exit_event) {
                Response::Handled => {},
                Response::Transition(_) => panic!(
                    "Do not perform transition on exit event."),
                _ => {}
            }
        }

        // Execute the entry path into the target state
        let entry_event = Self::Event::new_on_entry();
        for entry_state in entry_path.into_iter().rev() {
            match (entry_state)(self, &entry_event) {
                Response::Handled => {},
                Response::Transition(_) => panic!(
                    "Do not perform transition on entry event."),
                _ => {}
            }
        }
        *self.state_mut() = target;
    }
  
}