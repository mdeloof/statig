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
//! #![feature(generic_associated_types)]
//!
//! use stateful::{
//!     Response::{Handled, Super, Transition},
//!     Stateful, StateMachine
//! };
//!
//! // The response that will be returned by the state handlers.
//! type Result = stateful::Result<State>;
//!
//! // Define your event type.
//! enum Event {
//!     TimerElapsed,
//!     ButtonPressed,
//! }
//!
//! // Define your data type.
//! struct Blinky {
//!     light: bool,
//! }
//!
//! // Implement the `Stateful` trait.
//! impl stateful::Stateful for Blinky {
//!     // The state enum.
//!     type State = State;
//!     
//!     type Input = Event;
//!
//!     // The initial state of the state machine
//!     const INIT_STATE: State = State::On {};
//! }
//!
//! // Every state is a function. The `state_machine` macro derives an enum
//! // with variants for every state handler. The impl block with this
//! // attribute should only contain state handlers.
//! #[stateful::state_machine]
//! // Name the state enum.
//! impl Blinky {
//!     #[action]
//!     fn enter_on(&mut self) {
//!         self.light = true;
//!         println!("On");
//!     }
//!     
//!     // The state handler `on` has `Playing` as a parent state. Every
//!     // time we enter this state we want to call the method `enter_on`.
//!     #[state(superstate = "playing", entry_action = "enter_on")]
//!     fn on(&mut self, input: &Event) -> Result {
//!         match input {
//!             // When the event `TimerElapsed` is received, transition to
//!             // state `Off`.
//!             Event::TimerElapsed => Ok(Transition(State::Off {})),
//!             _ => Ok(Super),
//!         }
//!     }
//!
//!     #[action]
//!     fn enter_off(&mut self) {
//!         self.light = false;
//!         println!("Off");
//!     }
//!
//!     #[state(superstate = "playing", entry_action = "enter_off")]
//!     fn off(&mut self, input: &Event) -> Result {
//!         match input {
//!             Event::TimerElapsed => Ok(Transition(State::On {})),
//!             _ => Ok(Super),
//!         }
//!     }
//!     
//!     // The `derive_state` macro will take the snake_case name and convert
//!     // it to PascalCase to create the state variant. So `playing` becomes
//!     // `Playing`.
//!     #[superstate]
//!     fn playing(&mut self, input: &Event) -> Result {
//!         match input {
//!             Event::ButtonPressed => Ok(Transition(State::Paused {})),
//!             _ => Ok(Handled),
//!         }
//!     }
//!     
//!     #[action]
//!     fn enter_paused(&mut self) {
//!         println!("Paused");
//!     }
//!
//!     #[state(exit_action = "enter_paused")]
//!     fn paused(&mut self, input: &Event) -> Result {
//!         match input {
//!             Event::ButtonPressed => Ok(Transition(State::On {})),
//!             _ => Ok(Handled),
//!         }
//!     }
//! }
//!
//!     let mut state_machine = StateMachine::new(Blinky { light: false });
//!
//!     // Calling `init()` performs the transition into the initial state.
//!     state_machine.init();
//!
//!     for _ in 0..10 {
//!         // Dispatch an event to the state machine.
//!         state_machine.handle(&Event::TimerElapsed);
//!     }
//!
//!     state_machine.handle(&Event::ButtonPressed);
//!
//!     for _ in 0..10 {
//!         // The state machine is paused, so the `TimerElapsed` event does
//!         // not cause any transition.
//!         state_machine.handle(&Event::TimerElapsed);
//!     }
//!
//!     state_machine.handle(&Event::ButtonPressed);
//!
//!     for _ in 0..10 {
//!         state_machine.handle(&Event::TimerElapsed);
//!     }
//!
//! ```

#![no_std]
#![feature(generic_associated_types)]

use core::cmp::Ordering;

pub use stateful_macro::{action, state, state_machine, superstate};

pub trait Stateful {
    type State: State<Object = Self>;

    type Input;

    const INIT_STATE: Self::State;

    fn on_transition(&mut self, _source: &Self::State, _target: &Self::State) {}
}

pub type Result<S> = core::result::Result<Response<S>, Error<S>>;

pub enum Response<S> {
    Handled,
    Super,
    Transition(S),
}

pub enum Error<S> {
    Handled,
    Super,
    Transition(S),
}

pub trait ResultExt<T, S> {
    fn or_transition(self, state: S) -> core::result::Result<T, Error<S>>;

    fn or_handle(self) -> core::result::Result<T, Error<S>>;

    fn or_super(self) -> core::result::Result<T, Error<S>>;
}

impl<T, E, S> ResultExt<T, S> for core::result::Result<T, E> {
    fn or_transition(self, state: S) -> core::result::Result<T, Error<S>> {
        self.map_err(|_| Error::Transition(state))
    }

    fn or_handle(self) -> core::result::Result<T, Error<S>> {
        self.map_err(|_| Error::Handled)
    }

    fn or_super(self) -> core::result::Result<T, Error<S>> {
        self.map_err(|_| Error::Super)
    }
}

impl<T, S> ResultExt<T, S> for core::option::Option<T> {
    fn or_transition(self, state: S) -> core::result::Result<T, Error<S>> {
        self.ok_or(Error::Transition(state))
    }

    fn or_handle(self) -> core::result::Result<T, Error<S>> {
        self.ok_or(Error::Handled)
    }

    fn or_super(self) -> core::result::Result<T, Error<S>> {
        self.ok_or(Error::Super)
    }
}

pub struct StateMachine<O>
where
    O: Stateful,
{
    object: O,
    state: <O as Stateful>::State,
}

impl<O> StateMachine<O>
where
    O: Stateful,
{
    pub fn new(object: O) -> Self {
        Self {
            object,
            state: <O as Stateful>::INIT_STATE,
        }
    }

    pub fn state(&self) -> &<O as Stateful>::State {
        &self.state
    }

    /// # Safety
    /// Mutating the state externally could break the state machines internal
    /// invariants.
    pub unsafe fn state_mut(&mut self) -> &mut <O as Stateful>::State {
        &mut self.state
    }

    pub fn init(&mut self) {
        let enter_levels = self.state.depth();
        self.state.enter(&mut self.object, enter_levels);
    }

    pub fn handle(&mut self, event: &O::Input) {
        let result = self.state.handle(&mut self.object, event);
        match result {
            Ok(response) => match response {
                Response::Super => {}
                Response::Handled => {}
                Response::Transition(state) => self.transition(state),
            },
            Err(error) => match error {
                Error::Handled => {}
                Error::Super => {}
                Error::Transition(state) => self.transition(state),
            },
        }
    }

    pub fn transition(&mut self, mut target: <O as Stateful>::State) {
        // Get the transition path we need to perform from one state to the next.
        let (exit_levels, enter_levels) = self.state.transition_path(&mut target);

        // Perform the exit towards the common ancestor state.
        self.state.exit(&mut self.object, exit_levels);

        // Update the state.
        core::mem::swap(&mut self.state, &mut target);

        // Perform the enter actions.
        self.state.enter(&mut self.object, enter_levels);

        self.object.on_transition(&target, &self.state);
    }
}

impl<O> Default for StateMachine<O>
where
    O: Stateful + Default,
{
    fn default() -> Self {
        Self {
            object: <O as Default>::default(),
            state: <O as Stateful>::INIT_STATE,
        }
    }
}

impl<O> core::ops::Deref for StateMachine<O>
where
    O: Stateful,
{
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<O> core::ops::DerefMut for StateMachine<O>
where
    O: Stateful,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

pub trait State {
    type Superstate<'a>: Superstate<State = Self>
    where
        Self: 'a;
    type Object: Stateful<State = Self>;

    fn call_handler(
        &mut self,
        object: &mut Self::Object,
        input: &<Self::Object as Stateful>::Input,
    ) -> Result<Self>
    where
        Self: Sized;

    fn call_entry_action(&mut self, object: &mut Self::Object);

    fn call_exit_action(&mut self, object: &mut Self::Object);

    fn superstate(&mut self) -> Option<Self::Superstate<'_>>;

    fn same_state(&self, state: &Self) -> bool;

    fn depth(&mut self) -> usize {
        match self.superstate() {
            Some(mut superstate) => superstate.depth() + 1,
            None => 1,
        }
    }

    fn common_ancestor_depth(source: &mut Self, target: &mut Self) -> usize {
        if source.same_state(target) {
            return source.depth();
        }

        match (source.superstate(), target.superstate()) {
            (Some(source), Some(target)) => {
                <<Self as State>::Superstate<'_> as Superstate>::common_ancestor_depth(
                    source, target,
                )
            }
            _ => 0,
        }
    }

    fn transition_path(&mut self, target: &mut Self) -> (usize, usize) {
        if self.same_state(target) {
            return (1, 1);
        }

        let source_depth = self.depth();
        let target_depth = target.depth();

        if let (Some(source), Some(target)) = (self.superstate(), target.superstate()) {
            let common_state_depth = Self::Superstate::common_ancestor_depth(source, target);
            (
                source_depth - common_state_depth,
                target_depth - common_state_depth,
            )
        } else {
            (source_depth, target_depth)
        }
    }

    fn handle(
        &mut self,
        object: &mut Self::Object,
        event: &<Self::Object as Stateful>::Input,
    ) -> Result<Self>
    where
        Self: Sized,
    {
        match self.call_handler(object, event) {
            Ok(response) => match response {
                Response::Handled => Ok(Response::Handled),
                Response::Super => match self.superstate() {
                    Some(mut superstate) => superstate.handle(object, event),
                    None => Ok(Response::Super),
                },
                Response::Transition(state) => Ok(Response::Transition(state)),
            },
            Err(error) => match error {
                Error::Handled => Err(Error::Handled),
                Error::Super => match self.superstate() {
                    Some(mut superstate) => superstate.handle(object, event),
                    None => Err(Error::Super),
                },
                Error::Transition(state) => Err(Error::Transition(state)),
            },
        }
    }

    fn enter(&mut self, object: &mut Self::Object, levels: usize) {
        match levels {
            0 => (),
            1 => self.call_entry_action(object),
            _ => {
                if let Some(mut superstate) = self.superstate() {
                    superstate.enter(object, levels - 1);
                }
                self.call_entry_action(object);
            }
        }
    }

    fn exit(&mut self, object: &mut Self::Object, levels: usize) {
        match levels {
            0 => (),
            1 => self.call_exit_action(object),
            _ => {
                self.call_exit_action(object);
                if let Some(mut superstate) = self.superstate() {
                    superstate.exit(object, levels - 1);
                }
            }
        }
    }
}

pub trait Superstate
where
    Self: Sized,
{
    type State: State;

    fn call_handler(
        &mut self,
        object: &mut <Self::State as State>::Object,
        event: &<<Self::State as State>::Object as Stateful>::Input,
    ) -> Result<Self::State>;

    fn call_entry_action(&mut self, object: &mut <Self::State as State>::Object);

    fn call_exit_action(&mut self, object: &mut <Self::State as State>::Object);

    fn superstate(&mut self) -> Option<<Self::State as State>::Superstate<'_>>
    where
        Self: Sized;

    fn same_state(&self, state: &<Self::State as State>::Superstate<'_>) -> bool;

    fn depth(&mut self) -> usize {
        match self.superstate() {
            Some(mut superstate) => superstate.depth() + 1,
            None => 1,
        }
    }

    fn common_ancestor_depth<'a>(
        mut source: <Self::State as State>::Superstate<'_>,
        mut target: <Self::State as State>::Superstate<'_>,
    ) -> usize
    where
        Self: 'a,
    {
        match source.depth().cmp(&target.depth()) {
            Ordering::Equal => match source.same_state(&target) {
                true => source.depth(),
                false => match (source.superstate(), target.superstate()) {
                    (Some(source), Some(target)) => Self::common_ancestor_depth(source, target),
                    _ => 0,
                },
            },
            Ordering::Greater => match source.superstate() {
                Some(superstate) => Self::common_ancestor_depth(superstate, target),
                None => 0,
            },
            Ordering::Less => match target.superstate() {
                Some(superstate) => Self::common_ancestor_depth(source, superstate),
                None => 0,
            },
        }
    }

    fn handle(
        &mut self,
        object: &mut <Self::State as State>::Object,
        event: &<<Self::State as State>::Object as Stateful>::Input,
    ) -> Result<Self::State>
    where
        Self: Sized,
    {
        match self.call_handler(object, event) {
            Ok(response) => match response {
                Response::Handled => Ok(Response::Handled),
                Response::Super => match self.superstate() {
                    Some(mut superstate) => superstate.handle(object, event),
                    None => Ok(Response::Super),
                },
                Response::Transition(state) => Ok(Response::Transition(state)),
            },
            Err(error) => match error {
                Error::Handled => Err(Error::Handled),
                Error::Super => match self.superstate() {
                    Some(mut superstate) => superstate.handle(object, event),
                    None => Err(Error::Super),
                },
                Error::Transition(state) => Err(Error::Transition(state)),
            },
        }
    }

    fn enter(&mut self, object: &mut <Self::State as State>::Object, mut levels: usize) {
        match levels {
            0 => (),
            1 => self.call_entry_action(object),
            _ => {
                if let Some(mut superstate) = self.superstate() {
                    levels -= 1;
                    superstate.enter(object, levels);
                }
                self.call_entry_action(object);
            }
        }
    }

    fn exit(&mut self, object: &mut <Self::State as State>::Object, mut levels: usize) {
        match levels {
            0 => (),
            1 => self.call_exit_action(object),
            _ => {
                self.call_exit_action(object);
                if let Some(mut superstate) = self.superstate() {
                    levels -= 1;
                    superstate.exit(object, levels);
                }
            }
        }
    }
}
