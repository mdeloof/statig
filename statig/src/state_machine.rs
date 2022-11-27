use core::fmt::Debug;

use crate::Response;
use crate::State;
use crate::StateExt;
use crate::Superstate;

/// A data structure that declares the types associated with the state machine.
pub trait StateMachine
where
    Self: Sized,
{
    /// Event that is processed by the state machine.
    type Event<'a>;

    /// Enumeration of the various states.
    type State: State<Self>;

    /// Enumeration of the various superstates.
    type Superstate<'a>: Superstate<Self>
    where
        Self::State: 'a;

    /// Initial state of the state machine.
    const INITIAL: Self::State;

    /// Method that is called *before* an event is dispatched to a state or
    /// superstate handler.
    fn on_dispatch(&mut self, _state: StateOrSuperstate<'_, '_, Self>, _event: &Self::Event<'_>) {}

    /// Method that is called *after* every transition.
    fn on_transition(&mut self, _source: &Self::State, _target: &Self::State) {}
}

/// A state machine where the context is of type `Self`.
pub trait StateMachineContext: StateMachine {
    /// Create an uninitialized state machine. Use [UninitializedStateMachine::init] to initialize it.
    fn state_machine(self) -> UninitializedStateMachine<Self>
    where
        Self: Sized,
    {
        UninitializedStateMachine {
            context: self,
            state: Self::INITIAL,
        }
    }
}

impl<T> StateMachineContext for T where T: StateMachine {}

/// A state machine that has not yet been initialized.
///
/// A state machine needs to be initialized before it can handle events. This
/// can be done by calling the [`init`](Self::init) method on it. This will
/// execute all the entry actions into the initial state.
pub struct UninitializedStateMachine<O>
where
    O: StateMachine,
{
    context: O,
    state: <O as StateMachine>::State,
}

impl<O> UninitializedStateMachine<O>
where
    O: StateMachine,
{
    /// Initialize the state machine by excecuting all entry actions towards
    /// the initial state.
    ///
    /// ```
    /// # use statig::prelude::*;
    /// # #[derive(Default)]
    /// # pub struct Blinky {
    /// #     led: bool,
    /// # }
    /// #
    /// # pub struct Event;
    /// #
    /// # #[state_machine(initial = "State::on()")]
    /// # impl Blinky {
    /// #     #[state]
    /// #     fn on(event: &Event) -> Response<State> { Handled }
    /// # }
    /// #
    /// let uninitialized_state_machine = Blinky::default().state_machine();
    ///
    /// // The uninitialized state machine is consumed to create the initialized
    /// // state machine.
    /// let initialized_state_machine = uninitialized_state_machine.init();
    /// ```
    pub fn init(self) -> InitializedStatemachine<O> {
        let mut state_machine: InitializedStatemachine<O> = InitializedStatemachine {
            context: self.context,
            state: self.state,
        };
        state_machine.init();
        state_machine
    }
}

/// A state machine that has been initialized.
pub struct InitializedStatemachine<M>
where
    M: StateMachine,
{
    context: M,
    state: <M as StateMachine>::State,
}

impl<M> InitializedStatemachine<M>
where
    M: StateMachine,
{
    /// Get an immutable reference to the current state of the state machine.
    pub fn state(&self) -> &<M as StateMachine>::State {
        &self.state
    }

    /// Get a mutable reference the current state of the state machine.
    ///
    /// # Safety
    ///
    /// Mutating the state externally could break the state machines internal
    /// invariants.
    pub unsafe fn state_mut(&mut self) -> &mut <M as StateMachine>::State {
        &mut self.state
    }

    /// Handle the given event.
    pub fn handle(&mut self, event: &M::Event<'_>) {
        let response = self.state.handle(&mut self.context, event);

        match response {
            Response::Super => {}
            Response::Handled => {}
            Response::Transition(state) => self.transition(state),
        }
    }

    /// Initialize the state machine by excecuting all entry actions towards the initial state.
    fn init(&mut self) {
        let enter_levels = self.state.depth();
        self.state.enter(&mut self.context, enter_levels);
    }

    /// Transition from the current state to the given target state.
    fn transition(&mut self, mut target: <M as StateMachine>::State) {
        // Get the transition path we need to perform from one state to the next.
        let (exit_levels, enter_levels) = self.state.transition_path(&mut target);

        // Perform the exit from the previous state towards the common ancestor state.
        self.state.exit(&mut self.context, exit_levels);

        // Update the state.
        core::mem::swap(&mut self.state, &mut target);

        // Perform the entry actions from the common ancestor state into the new state.
        self.state.enter(&mut self.context, enter_levels);

        <M as StateMachine>::on_transition(&mut self.context, &target, &self.state);
    }
}

impl<'a, M> InitializedStatemachine<M>
where
    M: StateMachine<Event<'a> = ()>,
{
    /// This is the same as `handle(())` in the case `Event` is of type `()`.
    pub fn step(&mut self) {
        self.handle(&());
    }
}

impl<M> Default for InitializedStatemachine<M>
where
    M: StateMachine + Default,
{
    fn default() -> Self {
        Self {
            context: <M as Default>::default(),
            state: <M as StateMachine>::INITIAL,
        }
    }
}

impl<M> core::ops::Deref for InitializedStatemachine<M>
where
    M: StateMachine,
{
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<M> core::ops::DerefMut for InitializedStatemachine<M>
where
    M: StateMachine,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

/// Holds a reference to either a state or superstate.
pub enum StateOrSuperstate<'a, 'b, M: StateMachine>
where
    M::State: 'b,
{
    /// Reference to a state.
    State(&'a M::State),
    /// Reference to a superstate.
    Superstate(&'a M::Superstate<'b>),
}

impl<'a, 'b, M: StateMachine> core::fmt::Debug for StateOrSuperstate<'a, 'b, M>
where
    M::State: Debug,
    M::Superstate<'b>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::State(state) => f.debug_tuple("State").field(state as &dyn Debug).finish(),
            Self::Superstate(superstate) => f
                .debug_tuple("Superstate")
                .field(superstate as &dyn Debug)
                .finish(),
        }
    }
}
