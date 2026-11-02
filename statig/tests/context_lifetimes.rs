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
        state_machine.init_with_context(&mut super::Ctx {
            writer: &mut writer,
        });

        for event in events {
            state_machine.handle_with_context(
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
            .init_with_context(&mut super::Ctx {
                writer: &mut writer,
            })
            .await;

        for event in events {
            state_machine
                .handle_with_context(
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
