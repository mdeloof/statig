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
//!     Response::{Handled, Super, Transition},
//!     Stateful,
//! };
//!
//! // The response that will be returned by the state handlers.
//! type Response = stateful::Response<Blinky>;
//!
//! // Define your event type.
//! pub enum Event {
//!     TimerElapsed,
//!     ButtonPressed,
//! }
//!
//! // Define your data type.
//! pub struct Blinky {
//!     // The state field stores the state.
//!     state: State,
//!     
//!     // Your fields.
//!     light: bool,
//! }
//!
//! // Implement the `Stateful` trait.
//! impl stateful::Stateful for Blinky {
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
//! // Every state is a function. The `state_machine` macro derives an enum
//! // with variants for every state handler. The impl block with this
//! // attribute should only contain state handlers.
//! #[stateful::state_machine]
//! // Name the state enum.
//! #[state(name = "State")]
//! impl Blinky {
//!     
//!     // The state handler `on` has `Playing` as a parent state. Every
//!     // time we enter this state we want to call the method `enter_on`.
//!     #[state(superstate = "Playing", entry_action = "Blinky::enter_on")]
//!     fn on(&mut self, event: &Event) -> Response {
//!         match event {
//!             // When the event `TimerElapsed` is received, transition to
//!             // state `Off`.
//!             Event::TimerElapsed => Transition(State::Off),
//!             _ => Super,
//!         }
//!     }
//!
//!     #[state(superstate = "Playing", entry_action = "Blinky::enter_off")]
//!     fn off(&mut self, event: &Event) -> Response {
//!         match event {
//!             Event::TimerElapsed => Transition(State::On),
//!             _ => Super,
//!         }
//!     }
//!     
//!     // The `derive_state` macro will take the snake_case name and convert
//!     // it to PascalCase to create the state variant. So `playing` becomes
//!     // `Playing`.
//!     #[superstate]
//!     fn playing(&mut self, event: &Event) -> Response {
//!         match event {
//!             Event::ButtonPressed => Transition(State::Paused),
//!             _ => Handled,
//!         }
//!     }
//!
//!     #[state(exit_action = "Blinky::enter_paused")]
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
//!     // Calling `init()` performs the transition into the initial state.
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
//!         // The state machine is paused, so the `TimerElapsed` event does
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

//#![no_std]

use heapless::Vec;

pub use stateful_derive::*;

/// Type alias for state handlers.
pub type Handler<T, E> = fn(&mut T, &E) -> Response<T>;

/// Type alias for state and superstate actions.
pub type Action<T> = fn(&mut T);

/// The maximum depth states can be nested inside each other.
const DEPTH: usize = 16;

/// The response returned by a state handler function.
pub enum Response<T: Stateful> {
    /// The event has been handled.
    Handled,
    /// Defer the event to the parent state.
    Super,
    /// Transition to a leaf state.
    Transition(T::State),
}

/// A data structure that maintains an internal state, which affects the
/// way it handles events.
///
/// # Lifecycle
///
/// 1. `on_transition`

pub trait Stateful: Sized {
    /// The state enum that represents the various states of the state
    /// machine.
    type State: State<Object = Self>;

    /// The initial state of the state machine.
    const INIT_STATE: Self::State;

    /// Get a mutable reference to the current state.
    fn state_mut(&mut self) -> &mut Self::State;

    /// Handle an event from within the current state.
    fn handle(&mut self, event: &<Self::State as State>::Event) {
        let state = *self.state_mut();
        self.call_state_handler(state, event);
        self.on_event(event);
    }

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

    /// Callback that is called after the state machine has handled an
    /// event.
    fn on_event(&mut self, _event: &<Self::State as State>::Event) {}

    /// Callback that is called after the state machine has completed
    /// a transition.
    fn on_transition(
        &mut self,
        _source: &Self::State,
        _exit_path: &[<<Self as Stateful>::State as State>::Superstate],
        _entry_path: &[<<Self as Stateful>::State as State>::Superstate],
        _target: &Self::State,
    ) {
    }

    #[doc(hidden)]
    /// Transition from the outside into the given state.
    fn drill_into(&mut self, state: Self::State) {
        let entry_path = state.superstate_path();
        for state in entry_path.iter().rev() {
            if let Some(state_enter_handler) = state.entry_action() {
                state_enter_handler(self);
            }
        }
        if let Some(action) = state.entry_action() {
            action(self);
        }
    }

    #[doc(hidden)]
    /// Transition from the inner state to the outside.
    fn drill_out_of(&mut self, state: Self::State) {
        let exit_path = state.superstate_path();
        for state in exit_path.into_iter() {
            if let Some(state_enter_handler) = state.exit_action() {
                state_enter_handler(self);
            }
        }
        if let Some(action) = state.exit_action() {
            action(self);
        }
    }

    #[doc(hidden)]
    /// Handle an event from a given state.
    fn call_state_handler(&mut self, state: Self::State, event: &<Self::State as State>::Event) {
        match (state.handler())(self, event) {
            Response::Transition(target_state) => self.transition(target_state),
            Response::Super => match state.superstate() {
                Some(parent) => self.call_superstate_handler(parent, event),
                None => (),
            },
            Response::Handled => (),
        }
    }

    #[doc(hidden)]
    /// Handle an event from a given superstate.
    fn call_superstate_handler(
        &mut self,
        superstate: <<Self as Stateful>::State as State>::Superstate,
        event: &<Self::State as State>::Event,
    ) {
        match (superstate.handler())(self, event) {
            Response::Transition(target_state) => self.transition(target_state),
            Response::Super => match superstate.superstate() {
                Some(superstate) => self.call_superstate_handler(superstate, event),
                None => (),
            },
            Response::Handled => (),
        }
    }

    #[doc(hidden)]
    /// Perform a transition from the current state towards the target
    /// state.
    fn transition(&mut self, target: Self::State)
    where
        Self: Sized,
    {
        let source = *self.state_mut();

        let mut exit_path = source.superstate_path();
        let mut entry_path = target.superstate_path();

        // Starting from the root state, trim the entry and exit paths so
        // only uncommon states remain.
        while let (Some(&exit_temp), Some(&entry_temp)) = (exit_path.last(), entry_path.last()) {
            if exit_temp == entry_temp {
                exit_path.pop();
                entry_path.pop();
            } else {
                break;
            }
        }

        // Execute the exit action of the source state.
        if let Some(action) = source.exit_action() {
            action(self);
        }

        // Execute the exit actions along the exit path out of the source state.
        for state in exit_path.iter() {
            if let Some(action) = state.exit_action() {
                action(self);
            }
        }

        // Execute the entry actions along the entry path into the target state.
        entry_path.reverse();
        for state in entry_path.iter() {
            if let Some(action) = state.entry_action() {
                action(self);
            }
        }

        // Execute the entry action of the target state.
        if let Some(action) = target.entry_action() {
            action(self);
        }

        *self.state_mut() = target;

        // Call the `on_transition` callback.
        self.on_transition(&source, &exit_path, &entry_path, &target);
    }
}

/// Trait that should be implemented on the state enum.
pub trait State: Sized + Copy + PartialEq + std::fmt::Debug {
    /// The object on which the state handlers operate.
    type Object: Stateful<State = Self>;

    /// The event that is handled by the state handlers.
    type Event;

    /// Superate
    type Superstate: Superstate<Object = Self::Object, Event = Self::Event>;

    /// Get the associated state handler.
    fn handler(&self) -> Handler<Self::Object, Self::Event>;

    /// Get the superstate of the state, if defined.
    fn superstate(&self) -> Option<Self::Superstate> {
        None
    }

    /// Get the associated entry action, if defined.
    fn entry_action(&self) -> Option<Action<Self::Object>> {
        None
    }

    /// Get the associated exit action, if defined.
    fn exit_action(&self) -> Option<Action<Self::Object>> {
        None
    }

    /// Get the path from the state towards the root superstate.
    fn superstate_path(&self) -> Vec<Self::Superstate, DEPTH> {
        let mut path: Vec<Self::Superstate, DEPTH> = Vec::new();

        let mut temp_superstate = match self.superstate() {
            Some(superstate) => superstate,
            None => return path,
        };

        let _ = path.push(temp_superstate);
        let mut depth = 0;
        while let Some(superstate) = temp_superstate.superstate() {
            let _ = path.push(superstate);
            temp_superstate = superstate;
            depth += 1;

            if depth == DEPTH - 1 {
                panic!("reached max state nesting depth of {}", DEPTH)
            }
        }
        path
    }
}

pub trait Superstate: Sized + Copy + PartialEq + std::fmt::Debug {
    /// The object on which the state handlers operate.
    type Object: Stateful;

    /// The event that is handled by the state handlers.
    type Event;

    /// Get the associated state handler.
    fn handler(&self) -> Handler<Self::Object, Self::Event>;

    /// Get the superstate of the superstate, if it exists.
    fn superstate(&self) -> Option<Self> {
        None
    }

    /// Get the associated entry action, if defined.
    fn entry_action(&self) -> Option<Action<Self::Object>> {
        None
    }

    /// Get the associated exit action, if defined.
    fn exit_action(&self) -> Option<Action<Self::Object>> {
        None
    }
}
