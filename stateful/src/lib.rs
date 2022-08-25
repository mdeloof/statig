//! A Rust library to create hierarchical state machines. Every state is
//! function that handles an event or defers it to its parent state.
//!
//! ## Hierarchical State Machine
//!
//! A hierarchical state machine (HSM) is an extension of a traditional
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
//! In a hierarchical state machine we can add a parent state `Playing` that
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
//!     prelude::*
//! };
//!
//! // The response that will be returned by the state handlers.
//! type Response = stateful::Response<State>;
//!
//! // Define your event type.
//! pub enum Event {
//!     TimerElapsed,
//!     ButtonPressed,
//! }
//!
//! // Define your data type.
//! #[derive(Default)]
//! pub struct Blinky {
//!     light: bool,
//! }
//!
//! pub enum State {
//!     On,
//!     Off,
//!     Paused
//! }
//!
//! pub enum Superstate {
//!     Playing
//! }
//!
//! // Implement the `Stateful` trait.
//! impl StateMachine for Blinky {
//!     // The state enum.
//!     type State = State;
//!     
//!     // The superstate enum.
//!     type Superstate<'a> = Superstate;
//!     
//!     type Input = Event;
//!
//!     type Context = Self;
//!
//!     // The initial state of the state machine
//!     const INIT_STATE: State = State::On;
//! }
//!
//! impl stateful::State<Blinky> for State {
//!     fn call_handler(&mut self, blinky: &mut Blinky, input: &Event) -> Response {
//!         match self {
//!             State::On => blinky.on(input),
//!             State::Off => blinky.off(input),
//!             State::Paused => blinky.paused(input)
//!         }
//!     }
//!
//!     fn call_entry_action(&mut self, blinky: &mut Blinky) {
//!         match self {
//!             State::On => blinky.enter_on(),
//!             State::Off => blinky.enter_off(),
//!             _ => {}
//!         }
//!     }
//!
//!     fn superstate(&mut self) -> Option<Superstate> {
//!         match self {
//!             State::On => Some(Superstate::Playing),
//!             State::Off => Some(Superstate::Playing),
//!             State::Paused => None
//!         }
//!     }
//! }
//!
//! impl stateful::Superstate<Blinky> for Superstate {
//!     fn call_handler(&mut self, blinky: &mut Blinky, input: &Event) -> Response {
//!         match self {
//!             Superstate::Playing => blinky.playing(input)
//!         }
//!     }
//! }
//!
//! impl Blinky {
//!     fn enter_on(&mut self) {
//!         self.light = true;
//!         println!("On");
//!     }
//!     
//!     // The state handler `on` has `Playing` as a parent state. Every
//!     // time we enter this state we want to call the method `enter_on`.
//!     fn on(&mut self, input: &Event) -> Response {
//!         match input {
//!             // When the event `TimerElapsed` is received, transition to
//!             // state `Off`.
//!             Event::TimerElapsed => Transition(State::Off {}),
//!             _ => Super,
//!         }
//!     }
//!
//!     fn enter_off(&mut self) {
//!         self.light = false;
//!         println!("Off");
//!     }
//!
//!     fn off(&mut self, input: &Event) -> Response {
//!         match input {
//!             Event::TimerElapsed => Transition(State::On {}),
//!             _ => Super,
//!         }
//!     }
//!     
//!     fn playing(&mut self, input: &Event) -> Response {
//!         match input {
//!             Event::ButtonPressed => Transition(State::Paused {}),
//!             _ => Handled,
//!         }
//!     }
//!     
//!     fn enter_paused(&mut self) {
//!         println!("Paused");
//!     }
//!
//!     fn paused(&mut self, input: &Event) -> Response {
//!         match input {
//!             Event::ButtonPressed => Transition(State::On {}),
//!             _ => Handled,
//!         }
//!     }
//! }
//!
//! let mut state_machine = Blinky::state_machine().init();
//!
//! for _ in 0..10 {
//!     // Dispatch an event to the state machine.
//!     state_machine.handle(&Event::TimerElapsed);
//! }
//!
//! state_machine.handle(&Event::ButtonPressed);
//!
//! for _ in 0..10 {
//!     // The state machine is paused, so the `TimerElapsed` event does
//!     // not cause any transition.
//!     state_machine.handle(&Event::TimerElapsed);
//! }
//!
//! state_machine.handle(&Event::ButtonPressed);
//!
//! for _ in 0..10 {
//!     state_machine.handle(&Event::TimerElapsed);
//! }
//! ```

#![no_std]
#![feature(generic_associated_types)]

use core::cmp::Ordering;

pub mod prelude {
    pub use crate::Response::{self, *};
    pub use crate::{State, StateExt, StateMachine, StateMachineExt, Superstate, SuperstateExt};
}

pub trait StateMachine
where
    Self: Sized,
{
    type Context;

    type Input;

    type State: State<Self>;

    type Superstate<'a>: Superstate<Self>
    where
        Self::State: 'a;

    const INIT_STATE: Self::State;

    /// Method that is called *after* every transition.
    fn on_transition(_context: &mut Self::Context, _source: &Self::State, _target: &Self::State) {}
}

pub trait StateMachineExt: StateMachine {
    /// Create an uninitialized state machine. Use [UninitializedStateMachine::init] to initialize it.
    ///
    /// This methods assume the context implements [Default].
    fn state_machine() -> UninitializedStateMachine<Self>
    where
        Self: Sized,
        Self::Context: Default,
    {
        UninitializedStateMachine {
            context: Self::Context::default(),
            state: Self::INIT_STATE,
        }
    }

    /// Create an uninitialized state machine with a given context.
    fn with_context(context: Self::Context) -> UninitializedStateMachine<Self>
    where
        Self: Sized,
    {
        UninitializedStateMachine {
            context,
            state: Self::INIT_STATE,
        }
    }
}

impl<T> StateMachineExt for T where T: StateMachine {}

pub enum Response<S> {
    Handled,
    Super,
    Transition(S),
}

pub struct UninitializedStateMachine<O>
where
    O: StateMachine,
{
    context: O::Context,
    state: <O as StateMachine>::State,
}

impl<O> UninitializedStateMachine<O>
where
    O: StateMachine,
{
    pub fn init(self) -> InitializedStatemachine<O> {
        let mut state_machine: InitializedStatemachine<O> = InitializedStatemachine {
            context: self.context,
            state: self.state,
        };
        state_machine.init();
        state_machine
    }
}

pub struct InitializedStatemachine<O>
where
    O: StateMachine,
{
    context: O::Context,
    state: <O as StateMachine>::State,
}

impl<O> InitializedStatemachine<O>
where
    O: StateMachine,
{
    pub fn state(&self) -> &<O as StateMachine>::State {
        &self.state
    }

    /// # Safety
    ///
    /// Mutating the state externally could break the state machines internal
    /// invariants.
    pub unsafe fn state_mut(&mut self) -> &mut <O as StateMachine>::State {
        &mut self.state
    }

    pub fn handle(&mut self, event: &O::Input) {
        let response = self.state.handle(&mut self.context, event);
        match response {
            Response::Super => {}
            Response::Handled => {}
            Response::Transition(state) => self.transition(state),
        }
    }

    fn init(&mut self) {
        let enter_levels = self.state.depth();
        self.state.enter(&mut self.context, enter_levels);
    }

    fn transition(&mut self, mut target: <O as StateMachine>::State) {
        // Get the transition path we need to perform from one state to the next.
        let (exit_levels, enter_levels) = self.state.transition_path(&mut target);

        // Perform the exit from the previous state towards the common ancestor state.
        self.state.exit(&mut self.context, exit_levels);

        // Update the state.
        core::mem::swap(&mut self.state, &mut target);

        // Perform the entry actions from the common ancestor state into the new state.
        self.state.enter(&mut self.context, enter_levels);

        <O as StateMachine>::on_transition(&mut self.context, &target, &self.state);
    }
}

impl<O> Default for InitializedStatemachine<O>
where
    O: StateMachine,
    O::Context: Default,
{
    fn default() -> Self {
        Self {
            context: <<O as StateMachine>::Context as Default>::default(),
            state: <O as StateMachine>::INIT_STATE,
        }
    }
}

impl<O> core::ops::Deref for InitializedStatemachine<O>
where
    O: StateMachine,
{
    type Target = O::Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<O> core::ops::DerefMut for InitializedStatemachine<O>
where
    O: StateMachine,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

pub trait State<SM>
where
    Self: Sized,
    SM: StateMachine,
{
    fn call_handler(
        &mut self,
        _context: &mut <SM as StateMachine>::Context,
        _input: &<SM as StateMachine>::Input,
    ) -> Response<Self>;

    fn call_entry_action(&mut self, _context: &mut <SM as StateMachine>::Context) {}

    fn call_exit_action(&mut self, _context: &mut <SM as StateMachine>::Context) {}

    fn superstate(&mut self) -> Option<<SM as StateMachine>::Superstate<'_>> {
        None
    }
}

pub trait StateExt<SM>: State<SM>
where
    SM: StateMachine<State = Self>,
{
    fn same_state(lhs: &Self, rhs: &Self) -> bool {
        core::mem::discriminant(lhs) == core::mem::discriminant(rhs)
    }

    fn depth(&mut self) -> usize {
        match self.superstate() {
            Some(mut superstate) => superstate.depth() + 1,
            None => 1,
        }
    }

    fn common_ancestor_depth(source: &mut Self, target: &mut Self) -> usize {
        if Self::same_state(source, target) {
            return source.depth();
        }

        match (source.superstate(), target.superstate()) {
            (Some(source), Some(target)) => {
                <<SM as StateMachine>::Superstate<'_> as SuperstateExt<SM>>::common_ancestor_depth(
                    source, target,
                )
            }
            _ => 0,
        }
    }

    fn transition_path(&mut self, target: &mut Self) -> (usize, usize) {
        if Self::same_state(self, target) {
            return (1, 1);
        }

        let source_depth = self.depth();
        let target_depth = target.depth();

        if let (Some(source), Some(target)) = (self.superstate(), target.superstate()) {
            let common_state_depth =
                <SM as StateMachine>::Superstate::common_ancestor_depth(source, target);
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
        context: &mut <SM as StateMachine>::Context,
        event: &<SM as StateMachine>::Input,
    ) -> Response<Self>
    where
        Self: Sized,
    {
        let response = self.call_handler(context, event);
        match response {
            Response::Handled => Response::Handled,
            Response::Super => match self.superstate() {
                Some(mut superstate) => superstate.handle(context, event),
                None => Response::Super,
            },
            Response::Transition(state) => Response::Transition(state),
        }
    }

    fn enter(&mut self, context: &mut <SM as StateMachine>::Context, levels: usize) {
        match levels {
            0 => (),
            1 => self.call_entry_action(context),
            _ => {
                if let Some(mut superstate) = self.superstate() {
                    superstate.enter(context, levels - 1);
                }
                self.call_entry_action(context);
            }
        }
    }

    fn exit(&mut self, context: &mut <SM as StateMachine>::Context, levels: usize) {
        match levels {
            0 => (),
            1 => self.call_exit_action(context),
            _ => {
                self.call_exit_action(context);
                if let Some(mut superstate) = self.superstate() {
                    superstate.exit(context, levels - 1);
                }
            }
        }
    }
}

impl<T, SM> StateExt<SM> for T
where
    T: State<SM>,
    SM: StateMachine<State = T>,
{
}

pub trait Superstate<SM>
where
    SM: StateMachine,
{
    fn call_handler(
        &mut self,
        context: &mut <SM as StateMachine>::Context,
        event: &<SM as StateMachine>::Input,
    ) -> Response<<SM as StateMachine>::State>;

    fn call_entry_action(&mut self, _object: &mut <SM as StateMachine>::Context) {}

    fn call_exit_action(&mut self, _object: &mut <SM as StateMachine>::Context) {}

    fn superstate(&mut self) -> Option<<SM as StateMachine>::Superstate<'_>>
    where
        Self: Sized,
    {
        None
    }
}

impl<SM> Superstate<SM> for ()
where
    SM: StateMachine,
{
    fn call_handler(
        &mut self,
        _context: &mut <SM as StateMachine>::Context,
        _event: &<SM as StateMachine>::Input,
    ) -> Response<<SM as StateMachine>::State> {
        Response::Handled
    }

    fn call_entry_action(&mut self, _object: &mut <SM as StateMachine>::Context) {}

    fn call_exit_action(&mut self, _object: &mut <SM as StateMachine>::Context) {}

    fn superstate(&mut self) -> Option<<SM as StateMachine>::Superstate<'_>>
    where
        Self: Sized,
    {
        None
    }
}

pub trait SuperstateExt<SM>: Superstate<SM>
where
    SM: StateMachine,
    Self: Sized,
{
    fn same_state(
        lhs: &<SM as StateMachine>::Superstate<'_>,
        rhs: &<SM as StateMachine>::Superstate<'_>,
    ) -> bool {
        use core::mem::{discriminant, transmute_copy, Discriminant};

        // Generic associated types are invariant over any lifetime arguments, so the
        // compiler won't allow us to compare them directly. Instead we need to coerce them
        // to have the same lifetime by transmuting them to the same type.

        let lhs: Discriminant<<SM as StateMachine>::Superstate<'_>> =
            unsafe { transmute_copy(&discriminant(lhs)) };
        let rhs: Discriminant<<SM as StateMachine>::Superstate<'_>> =
            unsafe { transmute_copy(&discriminant(rhs)) };

        lhs == rhs
    }

    fn depth(&mut self) -> usize {
        match self.superstate() {
            Some(mut superstate) => superstate.depth() + 1,
            None => 1,
        }
    }

    fn common_ancestor_depth(
        mut source: <SM as StateMachine>::Superstate<'_>,
        mut target: <SM as StateMachine>::Superstate<'_>,
    ) -> usize {
        match source.depth().cmp(&target.depth()) {
            Ordering::Equal => match Self::same_state(&source, &target) {
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
        context: &mut <SM as StateMachine>::Context,
        event: &<SM as StateMachine>::Input,
    ) -> Response<<SM as StateMachine>::State>
    where
        Self: Sized,
    {
        let response = self.call_handler(context, event);
        match response {
            Response::Handled => Response::Handled,
            Response::Super => match self.superstate() {
                Some(mut superstate) => superstate.handle(context, event),
                None => Response::Super,
            },
            Response::Transition(state) => Response::Transition(state),
        }
    }

    fn enter(&mut self, context: &mut <SM as StateMachine>::Context, mut levels: usize) {
        match levels {
            0 => (),
            1 => self.call_entry_action(context),
            _ => {
                if let Some(mut superstate) = self.superstate() {
                    levels -= 1;
                    superstate.enter(context, levels);
                }
                self.call_entry_action(context);
            }
        }
    }

    fn exit(&mut self, context: &mut <SM as StateMachine>::Context, mut levels: usize) {
        match levels {
            0 => (),
            1 => self.call_exit_action(context),
            _ => {
                self.call_exit_action(context);
                if let Some(mut superstate) = self.superstate() {
                    levels -= 1;
                    superstate.exit(context, levels);
                }
            }
        }
    }
}

impl<T, SM> SuperstateExt<SM> for T
where
    T: Superstate<SM>,
    SM: StateMachine,
{
}
