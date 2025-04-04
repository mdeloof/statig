use core::fmt::Debug;

use crate::blocking::{Inner, IntoStateMachine, State, Superstate};

/// A state machine where the shared storage is of type `Self`.
pub trait IntoStateMachineExt: IntoStateMachine
where
    Self::State: State<Self>,
{
    /// Create a state machine that will be lazily initialized.
    fn state_machine(self) -> StateMachine<Self>
    where
        Self: Sized,
    {
        let inner = Inner {
            shared_storage: self,
            state: Self::initial(),
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
            state: Self::initial(),
        };
        UninitializedStateMachine { inner }
    }
}

impl<T> IntoStateMachineExt for T
where
    T: IntoStateMachine,
    Self::State: State<Self>,
{
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
    M::State: State<M>,
    for<'sub> M::Superstate<'sub>: Superstate<M>,
{
    /// Explicitly initialize the state machine. If the state machine is already initialized
    /// this is a no-op.
    pub fn init(&mut self)
    where
        for<'ctx> M: IntoStateMachine<Context<'ctx> = ()>,
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
    pub fn handle(&mut self, event: &M::Event<'_>)
    where
        for<'ctx> M: IntoStateMachine<Context<'ctx> = ()>,
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

    pub fn step(&mut self)
    where
        for<'evt, 'ctx> M: IntoStateMachine<Event<'evt> = (), Context<'ctx> = ()>,
    {
        self.handle_with_context(&(), &mut ());
    }

    pub fn step_with_context(&mut self, context: &mut M::Context<'_>)
    where
        for<'evt> M: IntoStateMachine<Event<'evt> = ()>,
    {
        self.handle_with_context(&(), context);
    }

    /// Get the current state.
    pub fn state(&self) -> &M::State {
        &self.inner.state
    }

    /// Get a reference to the [StateMachine]'s underlying type.
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
    /// # #[state_machine(
    /// #     initial = "State::on()",
    /// #     state(derive(Debug, PartialEq, Eq))
    /// # )]
    /// # impl Blinky {
    /// #     #[state]
    /// #     fn on(event: &Event) -> Response<State> { Handled }
    /// # }
    /// #
    /// let state_machine = Blinky::default().state_machine();
    /// assert_eq!(state_machine.inner().led, false);
    /// ```
    pub fn inner(&self) -> &M {
        &self.inner.shared_storage
    }

    /// Get a mutable reference to the [StateMachine]'s underlying type.
    ///
    /// # Safety
    ///
    /// - The user is responsible for validating that mutating a
    ///   [StateMachine] does not break any invariants.
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
    /// let mut state_machine = Blinky::default().state_machine();
    ///
    /// unsafe {
    ///     state_machine.inner_mut().led = true;
    /// }
    /// ```
    pub unsafe fn inner_mut(&mut self) -> &mut M {
        &mut self.inner.shared_storage
    }

    /// Get a mutable reference to the [StateMachine]'s current state.
    ///
    /// # Safety
    ///
    /// - The user is responsible for validating that mutating a
    ///   [StateMachine] does not break any invariants.
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
    /// let mut state_machine = Blinky::default().state_machine();
    ///
    /// unsafe {
    ///     *state_machine.state_mut() = State::on();
    /// }
    /// ```
    pub unsafe fn state_mut(&mut self) -> &mut M::State {
        &mut self.inner.state
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
            state: M::initial(),
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
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
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
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
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
#[cfg_attr(docsrs, doc(cfg(feature = "bevy")))]
impl<M> bevy_ecs::component::Component for StateMachine<M>
where
    Self: 'static + Send + Sync,
    M: IntoStateMachine,
{
    type Storage = bevy_ecs::component::TableStorage;
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
    M::State: State<M>,
{
    /// Handle the given event.
    pub fn handle(&mut self, event: &M::Event<'_>)
    where
        for<'ctx> M: IntoStateMachine<Context<'ctx> = ()>,
        for<'sub> M::Superstate<'sub>: Superstate<M>,
    {
        self.handle_with_context(event, &mut ());
    }

    /// Handle the given event.
    pub fn handle_with_context(&mut self, event: &M::Event<'_>, context: &mut M::Context<'_>)
    where
        M: IntoStateMachine,
        for<'sub> M::Superstate<'sub>: Superstate<M>,
    {
        self.inner.handle_with_context(event, context);
    }

    /// This is the same as `handle(())` in the case `Event` is of type `()`.
    pub fn step(&mut self)
    where
        for<'evt, 'ctx> M: IntoStateMachine<Event<'evt> = (), Context<'ctx> = ()>,
        for<'sub> M::Superstate<'sub>: Superstate<M>,
    {
        self.handle(&());
    }

    /// This is the same as `handle(())` in the case `Event` is of type `()`.
    pub fn step_with_context(&mut self, context: &mut M::Context<'_>)
    where
        for<'evt> M: IntoStateMachine<Event<'evt> = ()>,
        for<'sub> M::Superstate<'sub>: Superstate<M>,
    {
        self.handle_with_context(&(), context);
    }

    /// Get an immutable reference to the current state of the state machine.
    pub fn state(&self) -> &M::State {
        &self.inner.state
    }

    /// Get a reference to the [InitializedStateMachine]'s underlying type.
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
    /// # #[state_machine(
    /// #     initial = "State::on()",
    /// #     state(derive(Debug, PartialEq, Eq))
    /// # )]
    /// # impl Blinky {
    /// #     #[state]
    /// #     fn on(event: &Event) -> Response<State> { Handled }
    /// # }
    /// #
    /// # let uninitialized_state_machine = Blinky::default().uninitialized_state_machine();
    /// let initialized_state_machine = uninitialized_state_machine.init();
    /// assert_eq!(initialized_state_machine.inner().led, false);
    /// ```
    pub fn inner(&self) -> &M {
        &self.inner.shared_storage
    }

    /// Get a mutable reference to the [InitializedStateMachine]'s underlying type.
    ///
    /// # Safety
    ///
    /// - The user is responsible for validating that mutating a
    ///   [InitializedStateMachine] does not break any invariants.
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
    /// # let uninitialized_state_machine = Blinky::default().uninitialized_state_machine();
    /// let mut initialized_state_machine = uninitialized_state_machine.init();
    /// unsafe {
    ///     initialized_state_machine.inner_mut().led = true;
    /// }
    /// ```
    pub unsafe fn inner_mut(&mut self) -> &mut M {
        &mut self.inner.shared_storage
    }

    /// Get a mutable reference to the [InitializedStateMachine]'s current state.
    ///
    /// # Safety
    ///
    /// - The user is responsible for validating that mutating a
    ///   [StateMachine] does not break any invariants.
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
    /// let mut initialized_state_machine = uninitialized_state_machine.init();
    ///
    /// unsafe {
    ///     *initialized_state_machine.state_mut() = State::on();
    /// }
    /// ```
    pub unsafe fn state_mut(&mut self) -> &mut M::State {
        &mut self.inner.state
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
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
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
#[cfg_attr(docsrs, doc(cfg(feature = "bevy")))]
impl<M> bevy_ecs::component::Component for InitializedStateMachine<M>
where
    Self: 'static + Send + Sync,
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

impl<M> UninitializedStateMachine<M>
where
    M: IntoStateMachine,
    M::State: State<M>,
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
        for<'ctx> M: IntoStateMachine<Context<'ctx> = ()>,
        for<'sub> M::Superstate<'sub>: Superstate<M>,
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
    pub fn init_with_context(self, context: &mut M::Context<'_>) -> InitializedStateMachine<M>
    where
        for<'sub> M::Superstate<'sub>: Superstate<M>,
    {
        let mut state_machine = InitializedStateMachine { inner: self.inner };
        state_machine.inner.init_with_context(context);
        state_machine
    }

    /// Get a reference to the [UninitializedStateMachine]'s underlying type.
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
    /// # #[state_machine(
    /// #     initial = "State::on()",
    /// #     state(derive(Debug, PartialEq, Eq))
    /// # )]
    /// # impl Blinky {
    /// #     #[state]
    /// #     fn on(event: &Event) -> Response<State> { Handled }
    /// # }
    /// #
    /// let uninitialized_state_machine = Blinky::default().uninitialized_state_machine();
    ///
    /// assert_eq!(uninitialized_state_machine.inner().led, false);
    /// ```
    pub fn inner(&self) -> &M {
        &self.inner.shared_storage
    }

    /// Get a mutable reference to the [UninitializedStateMachine]'s underlying type.
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
    /// let mut uninitialized_state_machine = Blinky::default().uninitialized_state_machine();
    ///
    /// uninitialized_state_machine.inner_mut().led = true;
    /// ```
    pub fn inner_mut(&mut self) -> &mut M {
        &mut self.inner.shared_storage
    }

    /// Get a mutable reference to the [StateMachine]'s current state.
    ///
    /// # Safety
    ///
    /// - The user is responsible for validating that mutating a
    ///   [StateMachine] does not break any invariants.
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
    /// let mut uninitialized_state_machine = Blinky::default().uninitialized_state_machine();
    ///
    /// *uninitialized_state_machine.state_mut() = State::on();
    /// ```
    pub fn state_mut(&mut self) -> &mut M::State {
        &mut self.inner.state
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
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
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
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
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
