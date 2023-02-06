use core::fmt::Debug;

use crate::Response;
use crate::State;
use crate::StateExt;
use crate::Superstate;

/// A data structure that declares the types associated with the state machine.
pub trait IntoStateMachine
where
    Self: Sized,
{
    /// Event that is processed by the state machine.
    type Event<'a>;

    /// External context that can be passed in.
    type Context<'a>;

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
pub trait IntoStateMachineExt: IntoStateMachine {
    /// Create a state machine that will be lazily initialized.
    fn state_machine(self) -> StateMachine<Self>
    where
        Self: Sized,
    {
        let inner = Inner {
            shared_storage: self,
            state: Self::INITIAL,
        };
        StateMachine {
            inner,
            initialized: false,
        }
    }

    /// Create an uninitialized state machine that must be explicitly initialized with
    /// [`init`](UninitializedStateMachine::init).
    fn uninitialized_state_machine(self) -> UninitializedStateMachine<Self> {
        let inner = Inner {
            shared_storage: self,
            state: Self::INITIAL,
        };
        UninitializedStateMachine { inner }
    }
}

impl<T> IntoStateMachineExt for T where T: IntoStateMachine {}

/// Private internal representation of a state machine that is used for the public types.
struct Inner<M>
where
    M: IntoStateMachine,
{
    shared_storage: M,
    state: M::State,
}

impl<M> Inner<M>
where
    M: IntoStateMachine,
{
    /// Initialize the state machine by executing all entry actions towards the initial state.
    fn init_with_context(&mut self, context: &mut M::Context<'_>) {
        let enter_levels = self.state.depth();
        self.state
            .enter(&mut self.shared_storage, context, enter_levels);
    }

    /// Handle the given event.
    fn handle_with_context(&mut self, event: &M::Event<'_>, context: &mut M::Context<'_>) {
        let response = self.state.handle(&mut self.shared_storage, event, context);

        match response {
            Response::Super => {}
            Response::Handled => {}
            Response::Transition(state) => self.transition(state, context),
        }
    }

    /// Transition from the current state to the given target state.
    fn transition(&mut self, mut target: M::State, context: &mut M::Context<'_>) {
        // Get the transition path we need to perform from one state to the next.
        let (exit_levels, enter_levels) = self.state.transition_path(&mut target);

        // Perform the exit from the previous state towards the common ancestor state.
        self.state
            .exit(&mut self.shared_storage, context, exit_levels);

        // Update the state.
        core::mem::swap(&mut self.state, &mut target);

        // Perform the entry actions from the common ancestor state into the new state.
        self.state
            .enter(&mut self.shared_storage, context, enter_levels);

        M::ON_TRANSITION(&mut self.shared_storage, &target, &self.state);
    }
}

impl<M> Clone for Inner<M>
where
    M: IntoStateMachine + Clone,
    M::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            shared_storage: self.shared_storage.clone(),
            state: self.state.clone(),
        }
    }
}

impl<M> PartialEq for Inner<M>
where
    M: IntoStateMachine + PartialEq,
    M::State: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.shared_storage == other.shared_storage && self.state == other.state
    }
}

impl<M> Eq for Inner<M>
where
    M: IntoStateMachine + Eq,
    M::State: Eq,
{
}

#[cfg(feature = "serde")]
impl<M> serde::Serialize for Inner<M>
where
    M: IntoStateMachine + serde::Serialize,
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
/// A serialized state machine can only be deserialized into an [`UnInitializedStateMachine`] and can
/// then be initialized with [`init`](UnInitializedStateMachine::init).
impl<'de, M> serde::Deserialize<'de> for Inner<M>
where
    M: IntoStateMachine + serde::Deserialize<'de>,
    M::State: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::marker::PhantomData;

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

        struct InnerVisitor<M>(PhantomData<M>);

        impl<'de, M> serde::de::Visitor<'de> for InnerVisitor<M>
        where
            M: IntoStateMachine + serde::Deserialize<'de>,
            M::State: serde::Deserialize<'de>,
        {
            type Value = Inner<M>;

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
                let inner = Inner {
                    shared_storage,
                    state,
                };
                Ok(inner)
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
                let inner = Inner {
                    shared_storage,
                    state,
                };
                Ok(inner)
            }
        }

        const FIELDS: &[&str] = &["shared_storage", "state"];
        deserializer.deserialize_struct(
            "StateMachine",
            FIELDS,
            InnerVisitor(PhantomData::default()),
        )
    }
}

/// A state machine that will be lazily initialized.
pub struct StateMachine<M>
where
    M: IntoStateMachine,
{
    inner: Inner<M>,
    initialized: bool,
}

impl<M> StateMachine<M>
where
    M: IntoStateMachine,
{
    /// Explicitly initialize the state machine. If the state machine is already initialized
    /// this is a no-op.
    pub fn init<'a>(&mut self)
    where
        M: IntoStateMachine<Context<'a> = ()>,
    {
        self.init_with_context(&mut ());
    }

    /// Explicitly initialize the state machine. If the state machine is already initialized
    /// this is a no-op.
    pub fn init_with_context(&mut self, context: &mut M::Context<'_>) {
        if !self.initialized {
            self.inner.init_with_context(context);
            self.initialized = true;
        }
    }

    /// Handle an event. If the state machine is still uninitialized, it will be initialized
    /// before handling the event.
    pub fn handle<'a>(&mut self, event: &M::Event<'_>)
    where
        M: IntoStateMachine<Context<'a> = ()>,
    {
        self.handle_with_context(event, &mut ());
    }

    /// Handle an event. If the state machine is still uninitialized, it will be initialized
    /// before handling the event.
    pub fn handle_with_context(&mut self, event: &M::Event<'_>, context: &mut M::Context<'_>) {
        if !self.initialized {
            self.inner.init_with_context(context);
            self.initialized = true;
        }
        self.inner.handle_with_context(event, context);
    }

    pub fn step<'a>(&mut self)
    where
        M: IntoStateMachine<Event<'a> = (), Context<'a> = ()>,
    {
        self.handle_with_context(&(), &mut ());
    }

    pub fn step_with_context<'a>(&mut self, context: &mut M::Context<'a>)
    where
        M: IntoStateMachine<Event<'a> = ()>,
    {
        self.handle_with_context(&(), context);
    }

    /// Get the current state.
    pub fn state(&self) -> &M::State {
        &self.inner.state
    }
}

impl<M> Clone for StateMachine<M>
where
    M: IntoStateMachine + Clone,
    M::State: Clone,
{
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        let initialized = self.initialized;
        Self { inner, initialized }
    }
}

impl<M> PartialEq for StateMachine<M>
where
    M: IntoStateMachine + PartialEq,
    M::State: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner && self.initialized == other.initialized
    }
}

impl<M> Eq for StateMachine<M>
where
    M: IntoStateMachine + PartialEq,
    M::State: PartialEq,
{
}

impl<M> Default for StateMachine<M>
where
    M: IntoStateMachine + Default,
{
    fn default() -> Self {
        let inner = Inner {
            shared_storage: M::default(),
            state: M::INITIAL,
        };
        Self {
            inner,
            initialized: false,
        }
    }
}

impl<M> core::ops::Deref for StateMachine<M>
where
    M: IntoStateMachine,
{
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.inner.shared_storage
    }
}

#[cfg(feature = "serde")]
impl<M> serde::Serialize for StateMachine<M>
where
    M: IntoStateMachine + serde::Serialize,
    M::State: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
/// A serialized state machine can only be deserialized into an [`UnInitializedStateMachine`] and can
/// then be initialized with [`init`](UnInitializedStateMachine::init).
impl<'de, M> serde::Deserialize<'de> for StateMachine<M>
where
    M: IntoStateMachine + serde::Deserialize<'de>,
    M::State: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner: Inner<M> = Inner::deserialize(deserializer)?;
        Ok(StateMachine {
            inner,
            initialized: false,
        })
    }
}

#[cfg(feature = "bevy")]
impl<M> bevy_ecs::component::Component for StateMachine<M>
where
    Self: Send + Sync + 'static,
    M: IntoStateMachine,
{
    type Storage = bevy_ecs::component::TableStorage;
}

/// A state machine that has not yet been initialized.
///
/// A state machine needs to be initialized before it can handle events. This
/// can be done by calling the [`init`](Self::init) method on it. This will
/// execute all the entry actions into the initial state.
pub struct UninitializedStateMachine<M>
where
    M: IntoStateMachine,
{
    inner: Inner<M>,
}

impl<'a, M> UninitializedStateMachine<M>
where
    M: IntoStateMachine,
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
    /// let uninitialized_state_machine = Blinky::default().uninitialized_state_machine();
    ///
    /// // The uninitialized state machine is consumed to create the initialized
    /// // state machine.
    /// let initialized_state_machine = uninitialized_state_machine.init();
    /// ```
    pub fn init(self) -> InitializedStateMachine<M>
    where
        M: IntoStateMachine<Context<'a> = ()>,
    {
        let mut state_machine = InitializedStateMachine { inner: self.inner };
        state_machine.inner.init_with_context(&mut ());
        state_machine
    }

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
    /// let uninitialized_state_machine = Blinky::default().uninitialized_state_machine();
    ///
    /// // The uninitialized state machine is consumed to create the initialized
    /// // state machine.
    /// let initialized_state_machine = uninitialized_state_machine.init();
    /// ```
    pub fn init_with_context(self, context: &mut M::Context<'a>) -> InitializedStateMachine<M> {
        let mut state_machine = InitializedStateMachine { inner: self.inner };
        state_machine.inner.init_with_context(context);
        state_machine
    }
}

impl<M> Clone for UninitializedStateMachine<M>
where
    M: IntoStateMachine + Clone,
    M::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<M> Debug for UninitializedStateMachine<M>
where
    M: IntoStateMachine + Debug,
    M::State: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UnInitializedStateMachine")
            .field("shared_storage", &self.inner.shared_storage as &dyn Debug)
            .field("state", &self.inner.state as &dyn Debug)
            .finish()
    }
}

impl<M> PartialEq for UninitializedStateMachine<M>
where
    M: IntoStateMachine + PartialEq,
    M::State: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<M> Eq for UninitializedStateMachine<M>
where
    M: IntoStateMachine + PartialEq + Eq,
    M::State: PartialEq + Eq,
{
}

impl<M> core::ops::Deref for UninitializedStateMachine<M>
where
    M: IntoStateMachine,
{
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.inner.shared_storage
    }
}

#[cfg(feature = "serde")]
impl<M> serde::Serialize for UninitializedStateMachine<M>
where
    M: IntoStateMachine + serde::Serialize,
    M::State: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
/// A serialized state machine can only be deserialized into an [`UnInitializedStateMachine`] and can
/// then be initialized with [`init`](UnInitializedStateMachine::init).
impl<'de, M> serde::Deserialize<'de> for UninitializedStateMachine<M>
where
    M: IntoStateMachine + serde::Deserialize<'de>,
    M::State: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner: Inner<M> = Inner::deserialize(deserializer)?;
        Ok(UninitializedStateMachine { inner })
    }
}

/// A state machine that has been initialized.
pub struct InitializedStateMachine<M>
where
    M: IntoStateMachine,
{
    inner: Inner<M>,
}

impl<M> InitializedStateMachine<M>
where
    M: IntoStateMachine,
{
    /// Handle the given event.
    pub fn handle<'a>(&mut self, event: &M::Event<'_>)
    where
        M: IntoStateMachine<Context<'a> = ()>,
    {
        self.handle_with_context(event, &mut ());
    }

    /// Handle the given event.
    pub fn handle_with_context(&mut self, event: &M::Event<'_>, context: &mut M::Context<'_>) {
        self.inner.handle_with_context(event, context);
    }

    /// This is the same as `handle(())` in the case `Event` is of type `()`.
    pub fn step<'a>(&mut self)
    where
        M: IntoStateMachine<Event<'a> = (), Context<'a> = ()>,
    {
        self.handle(&());
    }

    /// This is the same as `handle(())` in the case `Event` is of type `()`.
    pub fn step_with_context<'a>(&mut self, context: &mut M::Context<'_>)
    where
        M: IntoStateMachine<Event<'a> = ()>,
    {
        self.handle_with_context(&(), context);
    }

    /// Get an immutable reference to the current state of the state machine.
    pub fn state(&self) -> &M::State {
        &self.inner.state
    }
}

impl<M> Clone for InitializedStateMachine<M>
where
    M: IntoStateMachine + Clone,
    M::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<M> Debug for InitializedStateMachine<M>
where
    M: IntoStateMachine + Debug,
    M::State: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InitializedStateMachine")
            .field("shared_storage", &self.inner.shared_storage as &dyn Debug)
            .field("state", &self.inner.state as &dyn Debug)
            .finish()
    }
}

impl<M> PartialEq for InitializedStateMachine<M>
where
    M: IntoStateMachine + PartialEq,
    M::State: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<M> Eq for InitializedStateMachine<M>
where
    M: IntoStateMachine + PartialEq + Eq,
    M::State: PartialEq + Eq,
{
}

impl<M> core::ops::Deref for InitializedStateMachine<M>
where
    M: IntoStateMachine,
{
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.inner.shared_storage
    }
}

#[cfg(feature = "serde")]
/// Once an [`InitializedStateMachine`] is serialized, it can only be deserialized into
/// an [`UnInitializedStateMachine`] which can then be re-initialized with the
/// [`init`](UnInitializedStateMachine::init) method.
impl<M> serde::Serialize for InitializedStateMachine<M>
where
    M: IntoStateMachine + serde::Serialize,
    M::State: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut serializer = serializer.serialize_struct("StateMachine", 2)?;
        serializer.serialize_field("shared_storage", &self.inner.shared_storage)?;
        serializer.serialize_field("state", &self.inner.state)?;
        serializer.end()
    }
}

#[cfg(feature = "bevy")]
impl<M> bevy_ecs::component::Component for InitializedStateMachine<M>
where
    Self: Send + Sync + 'static,
    M: IntoStateMachine,
{
    type Storage = bevy_ecs::component::TableStorage;
}

/// Holds a reference to either a state or superstate.
pub enum StateOrSuperstate<'a, 'b, M: IntoStateMachine>
where
    M::State: 'b,
{
    /// Reference to a state.
    State(&'a M::State),
    /// Reference to a superstate.
    Superstate(&'a M::Superstate<'b>),
}

impl<'a, 'b, M: IntoStateMachine> core::fmt::Debug for StateOrSuperstate<'a, 'b, M>
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
    M: IntoStateMachine,
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
    M: IntoStateMachine + PartialEq + Eq,
    M::State: 'b + PartialEq + Eq,
    M::Superstate<'b>: PartialEq + Eq,
{
}
