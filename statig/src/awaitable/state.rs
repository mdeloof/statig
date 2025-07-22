use core::future::Future;

use crate::awaitable::{IntoStateMachine, Superstate, SuperstateExt};
use crate::Outcome;
use crate::StateOrSuperstate;

/// An enum that represents the leaf states of the state machine.
pub trait State<M>
where
    Self: Sized,
    M: IntoStateMachine<State = Self>,
    for<'b> M::Superstate<'b>: Superstate<M>,
{
    /// Call the handler for the current state and let it handle the given event.
    fn call_handler(
        &mut self,
        shared_storage: &mut M,
        event: &M::Event<'_>,
        context: &mut M::Context<'_>,
    ) -> impl Future<Output = Outcome<Self, M::Response>>;

    #[allow(unused)]
    /// Call the entry action for the current state.
    fn call_entry_action(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'_>,
    ) -> impl Future<Output = ()> {
        core::future::ready(())
    }

    #[allow(unused)]
    /// Call the exit action for the current state.
    fn call_exit_action(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'_>,
    ) -> impl Future<Output = ()> {
        core::future::ready(())
    }

    /// Return the superstate of the current state, if there is one.
    fn superstate(&mut self) -> Option<M::Superstate<'_>> {
        None
    }
}

/// Extensions for `State` trait.
pub trait StateExt<M>: State<M>
where
    M: IntoStateMachine<State = Self>,
    for<'b> M::Superstate<'b>: Superstate<M>,
{
    /// Check if two states are the same.
    fn same_state(lhs: &Self, rhs: &Self) -> bool {
        core::mem::discriminant(lhs) == core::mem::discriminant(rhs)
    }

    /// Get the depth of the current state.
    fn depth(&mut self) -> usize {
        match self.superstate() {
            Some(mut superstate) => superstate.depth() + 1,
            None => 1,
        }
    }

    /// Get the depth of the common ancestor of two states.
    fn common_ancestor_depth(source: &mut Self, target: &mut Self) -> usize {
        if Self::same_state(source, target) {
            return source.depth();
        }

        match (source.superstate(), target.superstate()) {
            (Some(source), Some(target)) => M::Superstate::common_ancestor_depth(source, target),
            _ => 0,
        }
    }

    /// Get the transition path that needs to be taken to get from the source state to
    /// the target state. The returned tuple respectively represents the levels that need
    /// to be exited, and the levels that need to be entered.
    fn transition_path(&mut self, target: &mut Self) -> (usize, usize) {
        if Self::same_state(self, target) {
            return (1, 1);
        }

        let source_depth = self.depth();
        let target_depth = target.depth();

        if let (Some(source), Some(target)) = (self.superstate(), target.superstate()) {
            let common_state_depth = M::Superstate::common_ancestor_depth(source, target);
            (
                source_depth - common_state_depth,
                target_depth - common_state_depth,
            )
        } else {
            (source_depth, target_depth)
        }
    }

    /// Handle the given event in the current state.

    fn handle(
        &mut self,
        shared_storage: &mut M,
        event: &M::Event<'_>,
        context: &mut M::Context<'_>,
    ) -> impl Future<Output = Outcome<Self, M::Response>> {
        async move {
            M::before_dispatch(
                shared_storage,
                StateOrSuperstate::State(self),
                event,
                context,
            )
            .await;

            let outcome = self.call_handler(shared_storage, event, context).await;

            M::after_dispatch(
                shared_storage,
                StateOrSuperstate::State(self),
                event,
                context,
            )
            .await;

            match outcome {
                Outcome::Handled(response) => Outcome::Handled(response),
                Outcome::Super => match self.superstate() {
                    Some(mut superstate) => loop {
                        M::before_dispatch(
                            shared_storage,
                            StateOrSuperstate::Superstate(&superstate),
                            event,
                            context,
                        )
                        .await;

                        let outcome = superstate
                            .call_handler(shared_storage, event, context)
                            .await;

                        M::after_dispatch(
                            shared_storage,
                            StateOrSuperstate::Superstate(&superstate),
                            event,
                            context,
                        )
                        .await;

                        match outcome {
                            Outcome::Handled(response) => break Outcome::Handled(response),
                            Outcome::Super => match superstate.superstate() {
                                Some(superstate_of_superstate) => {
                                    superstate = superstate_of_superstate
                                }
                                None => break Outcome::Handled(M::default_response()),
                            },
                            Outcome::Transition(state) => break Outcome::Transition(state),
                        }
                    },
                    None => Outcome::Super,
                },
                Outcome::Transition(state) => Outcome::Transition(state),
            }
        }
    }

    /// Starting from the current state, climb a given amount of levels and execute all the
    /// entry actions while going back down to the current state.
    fn enter(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'_>,
        mut levels: usize,
    ) -> impl Future<Output = ()> {
        async move {
            // For each level we need to enter, climb that number of levels and excecute
            // the superstate's entry action. We then decrement `levels` and again climb
            // up the tree and execute the next entry action. We keep doing this until
            // `levels` is equal to 1, which means the only entry action left to excecute
            // is the state's own.
            while levels > 1 {
                if let Some(mut superstate) = self.superstate() {
                    for _ in 2..levels {
                        superstate = match superstate.superstate() {
                            Some(superstate) => superstate,
                            None => break,
                        }
                    }
                    superstate.call_entry_action(shared_storage, context).await;
                }
                levels -= 1;
            }

            if levels == 1 {
                self.call_entry_action(shared_storage, context).await;
            }
        }
    }

    /// Starting from the current state, climb a given amount of levels and execute all the
    /// the exit actions while going up to a certain superstate.
    fn exit(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'_>,
        mut levels: usize,
    ) -> impl Future<Output = ()> {
        async move {
            // First we need to excucute that state's own exit action.
            if levels >= 1 {
                self.call_exit_action(shared_storage, context).await;
            }

            // For each level we need to exit, climb up one level in the tree and
            // execute the superstate's exit action, then decrement `levels`. As long as
            // `levels` is greater then 1, we keep climbing up the three and excecute
            // that superstate exit action.
            if let Some(mut superstate) = self.superstate() {
                while levels > 1 {
                    superstate.call_exit_action(shared_storage, context).await;
                    superstate = match superstate.superstate() {
                        Some(superstate) => superstate,
                        None => break,
                    };
                    levels -= 1;
                }
            }
        }
    }
}

impl<T, M> StateExt<M> for T
where
    Self: State<M>,
    M: IntoStateMachine<State = T>,
    for<'b> M::Superstate<'b>: Superstate<M>,
{
}
