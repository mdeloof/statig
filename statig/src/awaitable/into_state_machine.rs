use core::future::{self, Future};

use crate::StateOrSuperstate;

/// Trait for transforming a type into a state machine.
pub trait IntoStateMachine
where
    Self: Sized,
{
    /// Event that is processed by the state machine.
    type Event<'evt>;

    /// External context passed to state handlers, actions and hooks.
    ///
    /// The context takes two lifetimes. `'ctx` is the lifetime of the borrows held directly by
    /// the context value, and `'data` is the lifetime of the data behind those borrows. A
    /// context such as `Ctx<'ctx, 'data> { writer: &'ctx mut Writer<'data> }` is built fresh
    /// for every call to [`handle_with_context`], reborrowing a `Writer<'data>` that the caller
    /// owns for longer than any single call.
    ///
    /// A single lifetime cannot serve both roles once the context holds `&mut` to something
    /// that itself borrows. `&'a mut T` is invariant in `T`, so `&'a mut Writer<'a>` forces the
    /// short per-call reborrow and the long-lived data borrow to be the same lifetime, and the
    /// writer stays mutably borrowed until the data borrow ends. The second call in a loop is
    /// then rejected by the borrow checker.
    ///
    /// The bound `'data: 'ctx` makes `&'ctx mut Writer<'data>` well-formed. The trait declares
    /// it on this associated type and on every method that takes the context, so every
    /// override of such a method, for example [`State::call_handler`] or
    /// [`before_transition`](Self::before_transition), must declare `<'ctx, 'data: 'ctx>` as
    /// well. An override with elided or differently bounded lifetimes is rejected with E0195.
    /// An implementation that writes `type Context<'ctx, 'data> = ...;` without the bound is
    /// rejected with E0478.
    ///
    /// Overrides in this module return `impl Future`, and the returned future captures every
    /// lifetime in the method signature, including the elided ones. An override must therefore
    /// keep each elided lifetime of the trait's parameter types: write the event as
    /// `&<Machine as IntoStateMachine>::Event<'_>` (or `&Self::Event<'_>` inside
    /// `IntoStateMachine`) and the superstate as `Self::Superstate<'_>`, even when they resolve
    /// to types without lifetimes. Spelling the event as `&u32` drops a lifetime parameter and
    /// is rejected with E0195.
    ///
    /// A context with fewer lifetimes ignores the unused parameters:
    /// `type Context<'ctx, 'data: 'ctx> = Owned<'data>;` for a context that owns a value
    /// borrowing long-lived data, `= Borrowed<'ctx>;` for a context that only reborrows,
    /// and `= Plain;` or `= ();` for a context without lifetimes.
    ///
    /// [`handle_with_context`]: crate::awaitable::StateMachine::handle_with_context
    /// [`State::call_handler`]: crate::awaitable::State::call_handler
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::future::Future;
    /// #
    /// # use statig::awaitable::{self, IntoStateMachine, IntoStateMachineExt, StateMachine};
    /// # use statig::Outcome;
    /// #
    /// struct Writer<'w> {
    ///     log: &'w mut Vec<u32>,
    /// }
    ///
    /// struct Ctx<'a, 'b> {
    ///     writer: &'a mut Writer<'b>,
    /// }
    ///
    /// # #[derive(Default)]
    /// # struct Machine;
    /// #
    /// # enum State {
    /// #     Idle,
    /// # }
    /// #
    /// impl IntoStateMachine for Machine {
    ///     type Event<'evt> = u32;
    ///
    ///     type Context<'ctx, 'data: 'ctx> = Ctx<'ctx, 'data>;
    ///
    ///     // ...
    /// #     type State = State;
    /// #
    /// #     type Superstate<'sub> = ();
    /// #
    /// #     fn initial() -> Self::State {
    /// #         State::Idle
    /// #     }
    /// }
    ///
    /// impl awaitable::State<Machine> for State {
    ///     fn call_handler<'ctx, 'data: 'ctx>(
    ///         &mut self,
    ///         _machine: &mut Machine,
    ///         event: &<Machine as IntoStateMachine>::Event<'_>,
    ///         context: &mut Ctx<'ctx, 'data>,
    ///     ) -> impl Future<Output = Outcome<Self>> {
    ///         async move {
    ///             context.writer.log.push(*event);
    ///
    ///             Outcome::Handled
    ///         }
    ///     }
    /// }
    ///
    /// async fn drive(state_machine: &mut StateMachine<Machine>, mut writer: Writer<'_>) {
    ///     state_machine
    ///         .init_with_context(&mut Ctx { writer: &mut writer })
    ///         .await;
    ///
    ///     for event in [1, 2, 3] {
    ///         state_machine
    ///             .handle_with_context(&event, &mut Ctx { writer: &mut writer })
    ///             .await;
    ///     }
    /// }
    ///
    /// let mut log = Vec::new();
    /// let mut state_machine = Machine::default().state_machine();
    ///
    /// futures::executor::block_on(drive(&mut state_machine, Writer { log: &mut log }));
    ///
    /// assert_eq!(log, [1, 2, 3]);
    /// ```
    type Context<'ctx, 'data: 'ctx>;

    /// Enumeration of the various states.
    type State;

    /// Enumeration of the various superstates.
    type Superstate<'sub>
    where
        Self::State: 'sub;

    /// Constructor for the initial state of the state machine.
    fn initial() -> Self::State;

    /// Method that is called *before* an event is dispatched to a state or
    /// superstate handler.
    fn before_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        _state_or_superstate: StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
        _event: &Self::Event<'_>,
        _context: &mut Self::Context<'ctx, 'data>,
    ) -> impl Future<Output = ()> {
        future::ready(())
    }

    /// Method that is called *after* an event is dispatched to a state or
    /// superstate handler.
    fn after_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        _state_or_superstate: StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
        _event: &Self::Event<'_>,
        _context: &mut Self::Context<'ctx, 'data>,
    ) -> impl Future<Output = ()> {
        future::ready(())
    }

    /// Method that is called *before* every transition.
    fn before_transition<'ctx, 'data: 'ctx>(
        &mut self,
        _source: &Self::State,
        _target: &Self::State,
        _context: &mut Self::Context<'ctx, 'data>,
    ) -> impl Future<Output = ()> {
        future::ready(())
    }

    /// Method that is called *after* every transition.
    fn after_transition<'ctx, 'data: 'ctx>(
        &mut self,
        _source: &Self::State,
        _target: &Self::State,
        _context: &mut Self::Context<'ctx, 'data>,
    ) -> impl Future<Output = ()> {
        future::ready(())
    }
}

pub trait DispatchHook<'ctx, 'data: 'ctx, T, P>
where
    T: IntoStateMachine,
{
    fn call_dispatch_hook(
        self,
        shared_storage: &mut T,
        state_or_superstate: StateOrSuperstate<'_, T::State, T::Superstate<'_>>,
        event: &T::Event<'_>,
        context: &mut T::Context<'ctx, 'data>,
    ) -> impl Future<Output = ()>;
}

impl<'ctx, 'data: 'ctx, T, F> DispatchHook<'ctx, 'data, T, ((),)> for F
where
    T: IntoStateMachine,
    F: AsyncFnOnce(&mut T, StateOrSuperstate<'_, T::State, T::Superstate<'_>>, &T::Event<'_>),
{
    fn call_dispatch_hook(
        self,
        shared_storage: &mut T,
        state_or_superstate: StateOrSuperstate<'_, T::State, T::Superstate<'_>>,
        event: &T::Event<'_>,
        _context: &mut T::Context<'ctx, 'data>,
    ) -> impl Future<Output = ()> {
        (self)(shared_storage, state_or_superstate, event)
    }
}

impl<'ctx, 'data: 'ctx, T, F> DispatchHook<'ctx, 'data, T, ((), ())> for F
where
    T: IntoStateMachine,
    F: AsyncFnOnce(
        &mut T,
        StateOrSuperstate<'_, T::State, T::Superstate<'_>>,
        &T::Event<'_>,
        &mut T::Context<'ctx, 'data>,
    ),
{
    fn call_dispatch_hook(
        self,
        shared_storage: &mut T,
        state_or_superstate: StateOrSuperstate<'_, T::State, T::Superstate<'_>>,
        event: &T::Event<'_>,
        context: &mut T::Context<'ctx, 'data>,
    ) -> impl Future<Output = ()> {
        (self)(shared_storage, state_or_superstate, event, context)
    }
}

pub trait TransitionHook<'ctx, 'data: 'ctx, T, P>
where
    T: IntoStateMachine,
{
    fn call_transition_hook(
        self,
        shared_storage: &mut T,
        source: &T::State,
        target: &T::State,
        context: &mut T::Context<'ctx, 'data>,
    ) -> impl Future<Output = ()>;
}

impl<'ctx, 'data: 'ctx, T, F> TransitionHook<'ctx, 'data, T, ((),)> for F
where
    T: IntoStateMachine,
    F: AsyncFnOnce(&mut T, &T::State, &T::State),
{
    fn call_transition_hook(
        self,
        shared_storage: &mut T,
        source: &T::State,
        target: &T::State,
        _context: &mut T::Context<'ctx, 'data>,
    ) -> impl Future<Output = ()> {
        (self)(shared_storage, source, target)
    }
}

impl<'ctx, 'data: 'ctx, T, F> TransitionHook<'ctx, 'data, T, ((), ())> for F
where
    T: IntoStateMachine,
    F: AsyncFnOnce(&mut T, &T::State, &T::State, &mut T::Context<'ctx, 'data>),
{
    fn call_transition_hook(
        self,
        shared_storage: &mut T,
        source: &T::State,
        target: &T::State,
        context: &mut T::Context<'ctx, 'data>,
    ) -> impl Future<Output = ()> {
        (self)(shared_storage, source, target, context)
    }
}
