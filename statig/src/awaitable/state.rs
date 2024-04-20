use core::future::Future;

use crate::awaitable::{Superstate, SuperstateExt};
use crate::IntoStateMachine;
use crate::Response;
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
    ) -> impl Future<Output = Response<Self>>;

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
    ) -> impl Future<Output = Response<Self>> {
        async move {
            M::BEFORE_DISPATCH(shared_storage, StateOrSuperstate::State(self), event);

            let response = self.call_handler(shared_storage, event, context).await;

            M::AFTER_DISPATCH(shared_storage, StateOrSuperstate::State(self), event);

            match response {
                Response::Handled => Response::Handled,
                Response::Super => match self.superstate() {
                    Some(mut superstate) => {
                        M::BEFORE_DISPATCH(
                            shared_storage,
                            StateOrSuperstate::Superstate(&superstate),
                            event,
                        );

                        loop {
                            M::BEFORE_DISPATCH(
                                shared_storage,
                                StateOrSuperstate::Superstate(&superstate),
                                event,
                            );

                            let response = superstate
                                .call_handler(shared_storage, event, context)
                                .await;

                            M::AFTER_DISPATCH(
                                shared_storage,
                                StateOrSuperstate::Superstate(&superstate),
                                event,
                            );

                            match response {
                                Response::Handled => break Response::Handled,
                                Response::Super => match superstate.superstate() {
                                    Some(s) => superstate = s,
                                    None => break Response::Handled,
                                },
                                Response::Transition(state) => break Response::Transition(state),
                            }
                        }
                    }
                    None => Response::Super,
                },
                Response::Transition(state) => Response::Transition(state),
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
            if levels >= 1 {
                self.call_exit_action(shared_storage, context).await;
            }

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
