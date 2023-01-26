use core::fmt::Debug;
use core::marker::PhantomData;

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
    const ON_DISPATCH: fn(&mut Self, StateOrSuperstate<'_, '_, Self>, &Self::Event<'_>) =
        |_, _, _| {};

    /// Method that is called *after* every transition.
    const ON_TRANSITION: fn(&mut Self, &Self::State, &Self::State) = |_, _, _| {};
}

/// A state machine where the shared storage is of type `Self`.
pub trait StateMachineSharedStorage: StateMachine {
    /// Create an uninitialized state machine. Use [UnInitializedStateMachine::init] to initialize it.
    fn state_machine(self) -> UnInitializedStateMachine<Self>
    where
        Self: Sized,
    {
        UnInitializedStateMachine {
            shared_storage: self,
            state: Self::INITIAL,
        }
    }
}

impl<T> StateMachineSharedStorage for T where T: StateMachine {}

/// A state machine that has not yet been initialized.
///
/// A state machine needs to be initialized before it can handle events. This
/// can be done by calling the [`init`](Self::init) method on it. This will
/// execute all the entry actions into the initial state.
pub struct UnInitializedStateMachine<M>
where
    M: StateMachine,
{
    shared_storage: M,
    state: <M as StateMachine>::State,
}

impl<M> UnInitializedStateMachine<M>
where
    M: StateMachine,
{
    /// Initialize the state machine by executing all entry actions towards
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
    pub fn init(self) -> InitializedStateMachine<M> {
        let mut state_machine: InitializedStateMachine<M> = InitializedStateMachine {
            shared_storage: self.shared_storage,
            state: self.state,
        };
        state_machine.init();
        state_machine
    }
}

impl<M> Clone for UnInitializedStateMachine<M>
where
    M: StateMachine + Clone,
    <M as StateMachine>::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            shared_storage: self.shared_storage.clone(),
            state: self.state.clone(),
        }
    }
}

impl<M> Debug for UnInitializedStateMachine<M>
where
    M: StateMachine + Debug,
    M::State: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UnInitializedStateMachine")
            .field("shared_storage", &self.shared_storage as &dyn Debug)
            .field("state", &self.state as &dyn Debug)
            .finish()
    }
}

impl<M> PartialEq for UnInitializedStateMachine<M>
where
    M: StateMachine + PartialEq,
    M::State: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.shared_storage == other.shared_storage && self.state == other.state
    }
}

impl<M> Eq for UnInitializedStateMachine<M>
where
    M: StateMachine + PartialEq + Eq,
    M::State: PartialEq + Eq,
{
}

#[cfg(feature = "serde")]
impl<M> serde::Serialize for UnInitializedStateMachine<M>
where
    M: StateMachine + serde::Serialize,
    M::State: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut serializer = serializer.serialize_struct("StateMachine", 2)?;
        serializer.serialize_field("shared_storage", &self.shared_storage)?;
        serializer.serialize_field("state", &self.state)?;
        serializer.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, M> serde::Deserialize<'de> for UnInitializedStateMachine<M>
where
    M: StateMachine + serde::Deserialize<'de>,
    M::State: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        enum Field {
            SharedStorage,
            State,
        }

        impl<'de> serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                        formatter.write_str("`shared_storage` or `state`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "shared_storage" => Ok(Field::SharedStorage),
                            "state" => Ok(Field::State),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct UnInitializedStateMachineVisitor<M>(PhantomData<M>);

        impl<'de, M> serde::de::Visitor<'de> for UnInitializedStateMachineVisitor<M>
        where
            M: StateMachine + serde::Deserialize<'de>,
            M::State: serde::Deserialize<'de>,
        {
            type Value = UnInitializedStateMachine<M>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("StateMachine")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let shared_storage = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let state = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                Ok(UnInitializedStateMachine {
                    shared_storage,
                    state,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut shared_storage = None;
                let mut state = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::SharedStorage => {
                            if shared_storage.is_some() {
                                return Err(serde::de::Error::duplicate_field("shared_storage"));
                            }
                            shared_storage = Some(map.next_value()?);
                        }
                        Field::State => {
                            if state.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state = Some(map.next_value()?);
                        }
                    }
                }
                let shared_storage = shared_storage
                    .ok_or_else(|| serde::de::Error::missing_field("shared_storage"))?;
                let state = state.ok_or_else(|| serde::de::Error::missing_field("state"))?;
                Ok(UnInitializedStateMachine {
                    shared_storage,
                    state,
                })
            }
        }

        const FIELDS: &[&str] = &["shared_storage", "state"];
        deserializer.deserialize_struct(
            "StateMachine",
            FIELDS,
            UnInitializedStateMachineVisitor(PhantomData::default()),
        )
    }
}

/// A state machine that has been initialized.
pub struct InitializedStateMachine<M>
where
    M: StateMachine,
{
    shared_storage: M,
    state: <M as StateMachine>::State,
}

impl<M> InitializedStateMachine<M>
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
        let response = self.state.handle(&mut self.shared_storage, event);

        match response {
            Response::Super => {}
            Response::Handled => {}
            Response::Transition(state) => self.transition(state),
        }
    }

    /// Initialize the state machine by executing all entry actions towards the initial state.
    fn init(&mut self) {
        let enter_levels = self.state.depth();
        self.state.enter(&mut self.shared_storage, enter_levels);
    }

    /// Transition from the current state to the given target state.
    fn transition(&mut self, mut target: <M as StateMachine>::State) {
        // Get the transition path we need to perform from one state to the next.
        let (exit_levels, enter_levels) = self.state.transition_path(&mut target);

        // Perform the exit from the previous state towards the common ancestor state.
        self.state.exit(&mut self.shared_storage, exit_levels);

        // Update the state.
        core::mem::swap(&mut self.state, &mut target);

        // Perform the entry actions from the common ancestor state into the new state.
        self.state.enter(&mut self.shared_storage, enter_levels);

        <M as StateMachine>::ON_TRANSITION(&mut self.shared_storage, &target, &self.state);
    }
}

impl<'a, M> InitializedStateMachine<M>
where
    M: StateMachine<Event<'a> = ()>,
{
    /// This is the same as `handle(())` in the case `Event` is of type `()`.
    pub fn step(&mut self) {
        self.handle(&());
    }
}

impl<M> Clone for InitializedStateMachine<M>
where
    M: StateMachine + Clone,
    <M as StateMachine>::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            shared_storage: self.shared_storage.clone(),
            state: self.state.clone(),
        }
    }
}

impl<M> Debug for InitializedStateMachine<M>
where
    M: StateMachine + Debug,
    M::State: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InitializedStateMachine")
            .field("shared_storage", &self.shared_storage as &dyn Debug)
            .field("state", &self.state as &dyn Debug)
            .finish()
    }
}

impl<M> Default for InitializedStateMachine<M>
where
    M: StateMachine + Default,
{
    fn default() -> Self {
        Self {
            shared_storage: <M as Default>::default(),
            state: <M as StateMachine>::INITIAL,
        }
    }
}

impl<M> PartialEq for InitializedStateMachine<M>
where
    M: StateMachine + PartialEq,
    M::State: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.shared_storage == other.shared_storage && self.state == other.state
    }
}

impl<M> Eq for InitializedStateMachine<M>
where
    M: StateMachine + PartialEq + Eq,
    M::State: PartialEq + Eq,
{
}

impl<M> core::ops::Deref for InitializedStateMachine<M>
where
    M: StateMachine,
{
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.shared_storage
    }
}

impl<M> core::ops::DerefMut for InitializedStateMachine<M>
where
    M: StateMachine,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shared_storage
    }
}

#[cfg(feature = "serde")]
impl<M> serde::Serialize for InitializedStateMachine<M>
where
    M: StateMachine + serde::Serialize,
    M::State: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut serializer = serializer.serialize_struct("StateMachine", 2)?;
        serializer.serialize_field("shared_storage", &self.shared_storage)?;
        serializer.serialize_field("state", &self.state)?;
        serializer.end()
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

impl<'a, 'b, M> PartialEq for StateOrSuperstate<'a, 'b, M>
where
    M: StateMachine,
    M::State: 'b + PartialEq,
    M::Superstate<'b>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::State(state), Self::State(other)) => state == other,
            (Self::Superstate(superstate), Self::Superstate(other)) => superstate == other,
            _ => false,
        }
    }
}

impl<'a, 'b, M> Eq for StateOrSuperstate<'a, 'b, M>
where
    M: StateMachine + PartialEq + Eq,
    M::State: 'b + PartialEq + Eq,
    M::Superstate<'b>: PartialEq + Eq,
{
}
