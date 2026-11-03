#![allow(clippy::manual_async_fn)]

/// Caller-owned sink whose borrow of the log outlives every per-event context.
struct Writer<'w> {
    log: &'w mut Vec<u32>,
}

/// Context that reborrows a caller-owned [`Writer`] for the duration of a single call.
struct Ctx<'a, 'b> {
    writer: &'a mut Writer<'b>,
}

enum Event {
    Bump,
    Toggle,
}

#[cfg(test)]
mod blocking_manual {
    use statig::blocking::{self, IntoStateMachine, IntoStateMachineExt, StateMachine};
    use statig::Outcome;

    #[derive(Default)]
    struct Machine {
        dispatches: u32,
    }

    enum State {
        Idle,
        Active,
    }

    impl IntoStateMachine for Machine {
        type Event<'evt> = super::Event;

        type Context<'ctx, 'data: 'ctx> = super::Ctx<'ctx, 'data>;

        type State = State;

        type Superstate<'sub> = ();

        fn initial() -> Self::State {
            State::Idle
        }

        fn before_transition<'ctx, 'data: 'ctx>(
            &mut self,
            _source: &Self::State,
            _target: &Self::State,
            context: &mut Self::Context<'ctx, 'data>,
        ) {
            context.writer.log.push(100);
        }

        fn after_dispatch<'ctx, 'data: 'ctx>(
            &mut self,
            _state_or_superstate: statig::StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
            _event: &Self::Event<'_>,
            _context: &mut Self::Context<'ctx, 'data>,
        ) {
            self.dispatches += 1;
        }
    }

    impl blocking::State<Machine> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Machine,
            event: &<Machine as IntoStateMachine>::Event<'_>,
            context: &mut <Machine as IntoStateMachine>::Context<'ctx, 'data>,
        ) -> Outcome<Self> {
            match (self, event) {
                (State::Idle, super::Event::Bump) => {
                    context.writer.log.push(10);

                    Outcome::Handled
                }
                (State::Idle, super::Event::Toggle) => Outcome::Transition(State::Active),
                (State::Active, super::Event::Bump) => {
                    context.writer.log.push(20);

                    Outcome::Handled
                }
                (State::Active, super::Event::Toggle) => Outcome::Transition(State::Idle),
            }
        }

        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Machine,
            context: &mut <Machine as IntoStateMachine>::Context<'ctx, 'data>,
        ) {
            match self {
                State::Idle => context.writer.log.push(1),
                State::Active => context.writer.log.push(3),
            }
        }

        fn call_exit_action<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Machine,
            context: &mut <Machine as IntoStateMachine>::Context<'ctx, 'data>,
        ) {
            match self {
                State::Idle => context.writer.log.push(2),
                State::Active => context.writer.log.push(4),
            }
        }
    }

    fn drive(
        state_machine: &mut StateMachine<Machine>,
        mut writer: super::Writer<'_>,
        events: &[super::Event],
    ) {
        state_machine.init(&mut super::Ctx {
            writer: &mut writer,
        });

        for event in events {
            state_machine.handle(
                event,
                &mut super::Ctx {
                    writer: &mut writer,
                },
            );
        }
    }

    #[test]
    fn drives_two_lifetime_context_in_loop() {
        let mut log = Vec::new();
        let mut state_machine = Machine::default().state_machine();

        drive(
            &mut state_machine,
            super::Writer { log: &mut log },
            &[
                super::Event::Bump,
                super::Event::Toggle,
                super::Event::Bump,
                super::Event::Toggle,
            ],
        );

        assert_eq!(log, vec![1, 10, 100, 2, 3, 20, 100, 4, 1]);
        assert_eq!(state_machine.inner().dispatches, 4);
    }
}

#[cfg(test)]
#[cfg(feature = "async")]
mod awaitable_manual {
    use core::future::Future;

    use statig::awaitable::{self, IntoStateMachine, IntoStateMachineExt, StateMachine};
    use statig::Outcome;

    #[derive(Default)]
    struct Machine {
        dispatches: u32,
    }

    enum State {
        Idle,
        Active,
    }

    impl IntoStateMachine for Machine {
        type Event<'evt> = super::Event;

        type Context<'ctx, 'data: 'ctx> = super::Ctx<'ctx, 'data>;

        type State = State;

        type Superstate<'sub> = ();

        fn initial() -> Self::State {
            State::Idle
        }

        fn before_transition<'ctx, 'data: 'ctx>(
            &mut self,
            _source: &Self::State,
            _target: &Self::State,
            context: &mut Self::Context<'ctx, 'data>,
        ) -> impl Future<Output = ()> {
            async move {
                context.writer.log.push(100);
            }
        }

        fn after_dispatch<'ctx, 'data: 'ctx>(
            &mut self,
            _state_or_superstate: statig::StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
            _event: &Self::Event<'_>,
            _context: &mut Self::Context<'ctx, 'data>,
        ) -> impl Future<Output = ()> {
            async move {
                self.dispatches += 1;
            }
        }
    }

    impl awaitable::State<Machine> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Machine,
            event: &<Machine as IntoStateMachine>::Event<'_>,
            context: &mut <Machine as IntoStateMachine>::Context<'ctx, 'data>,
        ) -> impl Future<Output = Outcome<Self>> {
            async move {
                match (self, event) {
                    (State::Idle, super::Event::Bump) => {
                        context.writer.log.push(10);

                        Outcome::Handled
                    }
                    (State::Idle, super::Event::Toggle) => Outcome::Transition(State::Active),
                    (State::Active, super::Event::Bump) => {
                        context.writer.log.push(20);

                        Outcome::Handled
                    }
                    (State::Active, super::Event::Toggle) => Outcome::Transition(State::Idle),
                }
            }
        }

        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Machine,
            context: &mut <Machine as IntoStateMachine>::Context<'ctx, 'data>,
        ) -> impl Future<Output = ()> {
            async move {
                match self {
                    State::Idle => context.writer.log.push(1),
                    State::Active => context.writer.log.push(3),
                }
            }
        }

        fn call_exit_action<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Machine,
            context: &mut <Machine as IntoStateMachine>::Context<'ctx, 'data>,
        ) -> impl Future<Output = ()> {
            async move {
                match self {
                    State::Idle => context.writer.log.push(2),
                    State::Active => context.writer.log.push(4),
                }
            }
        }
    }

    async fn drive(
        state_machine: &mut StateMachine<Machine>,
        mut writer: super::Writer<'_>,
        events: &[super::Event],
    ) {
        state_machine
            .init(&mut super::Ctx {
                writer: &mut writer,
            })
            .await;

        for event in events {
            state_machine
                .handle(
                    event,
                    &mut super::Ctx {
                        writer: &mut writer,
                    },
                )
                .await;
        }
    }

    #[test]
    fn drives_two_lifetime_context_in_loop() {
        let mut log = Vec::new();
        let mut state_machine = Machine::default().state_machine();

        futures::executor::block_on(drive(
            &mut state_machine,
            super::Writer { log: &mut log },
            &[
                super::Event::Bump,
                super::Event::Toggle,
                super::Event::Bump,
                super::Event::Toggle,
            ],
        ));

        assert_eq!(log, vec![1, 10, 100, 2, 3, 20, 100, 4, 1]);
        assert_eq!(state_machine.inner().dispatches, 4);
    }
}

#[cfg(test)]
mod blocking_macro {
    use super::{Ctx, Event, Writer};
    use statig::prelude::*;

    #[derive(Default)]
    struct Machine {
        dispatches: u32,
    }

    #[state_machine(
        initial = "State::idle()",
        context = "Ctx<'ctx, 'data>",
        before_transition = "Self::before_transition",
        after_dispatch = "Self::after_dispatch"
    )]
    impl Machine {
        #[state(entry_action = "enter_idle", exit_action = "exit_idle")]
        fn idle(context: &mut Ctx<'_, '_>, event: &Event) -> Outcome<State> {
            match event {
                Event::Bump => {
                    context.writer.log.push(10);

                    Handled
                }
                Event::Toggle => Transition(State::active()),
            }
        }

        #[state(entry_action = "enter_active", exit_action = "exit_active")]
        fn active(context: &mut Ctx<'_, '_>, event: &Event) -> Outcome<State> {
            match event {
                Event::Bump => {
                    context.writer.log.push(20);

                    Handled
                }
                Event::Toggle => Transition(State::idle()),
            }
        }

        #[action]
        fn enter_idle(context: &mut Ctx<'_, '_>) {
            context.writer.log.push(1);
        }

        #[action]
        fn exit_idle(context: &mut Ctx<'_, '_>) {
            context.writer.log.push(2);
        }

        #[action]
        fn enter_active(context: &mut Ctx<'_, '_>) {
            context.writer.log.push(3);
        }

        #[action]
        fn exit_active(context: &mut Ctx<'_, '_>) {
            context.writer.log.push(4);
        }
    }

    impl Machine {
        fn before_transition(
            &mut self,
            _source: &State,
            _target: &State,
            context: &mut Ctx<'_, '_>,
        ) {
            context.writer.log.push(100);
        }

        fn after_dispatch(
            &mut self,
            _state_or_superstate: StateOrSuperstate<'_, State, Superstate>,
            _event: &Event,
        ) {
            self.dispatches += 1;
        }
    }

    fn drive(
        state_machine: &mut statig::blocking::StateMachine<Machine>,
        mut writer: Writer<'_>,
        events: &[Event],
    ) {
        state_machine.init(&mut Ctx {
            writer: &mut writer,
        });

        for event in events {
            state_machine.handle(
                event,
                &mut Ctx {
                    writer: &mut writer,
                },
            );
        }
    }

    #[test]
    fn drives_two_lifetime_context_in_loop() {
        let mut log = Vec::new();
        let mut state_machine = Machine::default().state_machine();

        drive(
            &mut state_machine,
            Writer { log: &mut log },
            &[Event::Bump, Event::Toggle, Event::Bump, Event::Toggle],
        );

        assert_eq!(log, vec![1, 10, 100, 2, 3, 20, 100, 4, 1]);
        assert_eq!(state_machine.inner().dispatches, 4);
    }
}

#[cfg(test)]
#[cfg(feature = "async")]
mod awaitable_macro {
    use super::{Ctx, Event, Writer};
    use statig::prelude::*;

    #[derive(Default)]
    struct Machine {
        dispatches: u32,
    }

    #[state_machine(
        initial = "State::idle()",
        context = "Ctx<'ctx, 'data>",
        before_transition = "Self::before_transition",
        after_dispatch = "Self::after_dispatch"
    )]
    impl Machine {
        #[state(entry_action = "enter_idle", exit_action = "exit_idle")]
        async fn idle(context: &mut Ctx<'_, '_>, event: &Event) -> Outcome<State> {
            match event {
                Event::Bump => {
                    context.writer.log.push(10);

                    Handled
                }
                Event::Toggle => Transition(State::active()),
            }
        }

        #[state(entry_action = "enter_active", exit_action = "exit_active")]
        async fn active(context: &mut Ctx<'_, '_>, event: &Event) -> Outcome<State> {
            match event {
                Event::Bump => {
                    context.writer.log.push(20);

                    Handled
                }
                Event::Toggle => Transition(State::idle()),
            }
        }

        #[action]
        async fn enter_idle(context: &mut Ctx<'_, '_>) {
            context.writer.log.push(1);
        }

        #[action]
        async fn exit_idle(context: &mut Ctx<'_, '_>) {
            context.writer.log.push(2);
        }

        #[action]
        async fn enter_active(context: &mut Ctx<'_, '_>) {
            context.writer.log.push(3);
        }

        #[action]
        async fn exit_active(context: &mut Ctx<'_, '_>) {
            context.writer.log.push(4);
        }
    }

    impl Machine {
        async fn before_transition(
            &mut self,
            _source: &State,
            _target: &State,
            context: &mut Ctx<'_, '_>,
        ) {
            context.writer.log.push(100);
        }

        async fn after_dispatch(
            &mut self,
            _state_or_superstate: StateOrSuperstate<'_, State, Superstate>,
            _event: &Event,
        ) {
            self.dispatches += 1;
        }
    }

    async fn drive(
        state_machine: &mut statig::awaitable::StateMachine<Machine>,
        mut writer: Writer<'_>,
        events: &[Event],
    ) {
        state_machine
            .init(&mut Ctx {
                writer: &mut writer,
            })
            .await;

        for event in events {
            state_machine
                .handle(
                    event,
                    &mut Ctx {
                        writer: &mut writer,
                    },
                )
                .await;
        }
    }

    #[test]
    fn drives_two_lifetime_context_in_loop() {
        let mut log = Vec::new();
        let mut state_machine = Machine::default().state_machine();

        futures::executor::block_on(drive(
            &mut state_machine,
            Writer { log: &mut log },
            &[Event::Bump, Event::Toggle, Event::Bump, Event::Toggle],
        ));

        assert_eq!(log, vec![1, 10, 100, 2, 3, 20, 100, 4, 1]);
        assert_eq!(state_machine.inner().dispatches, 4);
    }
}

#[cfg(test)]
mod owned_context {
    use super::{Event, Writer};
    use statig::prelude::*;

    /// Context that owns a [`Writer`], so only the data lifetime is named.
    struct Owned<'d> {
        writer: Writer<'d>,
    }

    #[derive(Default)]
    struct Machine;

    #[state_machine(initial = "State::idle()", context = "Owned<'data>")]
    impl Machine {
        #[state(entry_action = "enter_idle")]
        fn idle(context: &mut Owned<'_>, event: &Event) -> Outcome<State> {
            match event {
                Event::Bump => {
                    context.writer.log.push(10);

                    Handled
                }
                Event::Toggle => Transition(State::idle()),
            }
        }

        #[action]
        fn enter_idle(context: &mut Owned<'_>) {
            context.writer.log.push(1);
        }
    }

    fn drive(
        state_machine: &mut statig::blocking::StateMachine<Machine>,
        writer: Writer<'_>,
        events: &[Event],
    ) {
        let mut owned = Owned { writer };

        state_machine.init(&mut owned);

        for event in events {
            state_machine.handle(event, &mut owned);
        }
    }

    #[test]
    fn drives_owned_context_in_loop() {
        let mut log = Vec::new();
        let mut state_machine = Machine.state_machine();

        drive(
            &mut state_machine,
            Writer { log: &mut log },
            &[Event::Bump, Event::Bump, Event::Toggle, Event::Bump],
        );

        assert_eq!(log, vec![1, 10, 10, 1, 10]);
    }
}

#[cfg(test)]
mod borrowed_context {
    use super::Event;
    use statig::prelude::*;

    /// Context that borrows the log for a single call, so only the context lifetime is named.
    struct Borrowed<'b> {
        log: &'b mut Vec<u32>,
    }

    #[derive(Default)]
    struct Machine;

    #[state_machine(initial = "State::idle()", context = "Borrowed<'ctx>")]
    impl Machine {
        #[state(entry_action = "enter_idle")]
        fn idle(context: &mut Borrowed<'_>, event: &Event) -> Outcome<State> {
            match event {
                Event::Bump => {
                    context.log.push(10);

                    Handled
                }
                Event::Toggle => Transition(State::idle()),
            }
        }

        #[action]
        fn enter_idle(context: &mut Borrowed<'_>) {
            context.log.push(1);
        }
    }

    fn drive(
        state_machine: &mut statig::blocking::StateMachine<Machine>,
        log: &mut Vec<u32>,
        events: &[Event],
    ) {
        state_machine.init(&mut Borrowed { log: &mut *log });

        for event in events {
            state_machine.handle(event, &mut Borrowed { log: &mut *log });
        }
    }

    #[test]
    fn drives_borrowed_context_in_loop() {
        let mut log = Vec::new();
        let mut state_machine = Machine.state_machine();

        drive(
            &mut state_machine,
            &mut log,
            &[Event::Bump, Event::Bump, Event::Toggle, Event::Bump],
        );

        assert_eq!(log, vec![1, 10, 10, 1, 10]);
    }
}
