extern crate alloc;
use alloc::boxed::Box;

use core::future::Future;
use core::pin::Pin;

use crate::awaitable::{Superstate, SuperstateExt};
use crate::IntoStateMachine;
use crate::Response;
use crate::StateOrSuperstate;

/// An enum that represents the leaf states of the state machine.
pub trait State<M>
where
    Self: Sized + Send,
    M: IntoStateMachine<State = Self> + Send,
    for<'b> M::Superstate<'b>: Superstate<M> + Send,
{
    /// Call the handler for the current state and let it handle the given event.
    fn call_handler<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        event: &'fut M::Event<'_>,
        context: &'fut mut M::Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Response<Self>> + 'fut + Send>>;

    #[allow(unused)]
    /// Call the entry action for the current state.
    fn call_entry_action<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        context: &'fut mut M::Context<'_>,
    ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
        Box::pin(core::future::ready(()))
    }

    #[allow(unused)]
    /// Call the exit action for the current state.
    fn call_exit_action<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        context: &'fut mut M::Context<'_>,
    ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
        Box::pin(core::future::ready(()))
    }

    /// Return the superstate of the current state, if there is one.
    fn superstate(&mut self) -> Option<M::Superstate<'_>> {
        None
    }
}

/// Extensions for `State` trait.
pub trait StateExt<M>: State<M>
where
    Self: Send,
    M: IntoStateMachine<State = Self> + Send,
    for<'evt> M::Event<'evt>: Send + Sync,
    for<'ctx> M::Context<'ctx>: Send + Sync,
    for<'b> M::Superstate<'b>: Superstate<M> + Send,
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
    fn handle<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        event: &'fut M::Event<'_>,
        context: &'fut mut M::Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Response<Self>> + 'fut + Send>> {
        let future = async move {
            M::ON_DISPATCH(shared_storage, StateOrSuperstate::State(self), event);

            let response = self.call_handler(shared_storage, event, context).await;

            match response {
                Response::Handled => Response::Handled,
                Response::Super => match self.superstate() {
                    Some(mut superstate) => {
                        M::ON_DISPATCH(
                            shared_storage,
                            StateOrSuperstate::Superstate(&superstate),
                            event,
                        );

                        superstate.handle(shared_storage, event, context).await
                    }
                    None => Response::Super,
                },
                Response::Transition(state) => Response::Transition(state),
            }
        };
        Box::pin(future)
    }

    /// Starting from the current state, climb a given amount of levels and execute all the
    /// entry actions while going back down to the current state.
    fn enter<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        context: &'fut mut M::Context<'_>,
        levels: usize,
    ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
        let future = async move {
            match levels {
                0 => (),
                1 => self.call_entry_action(shared_storage, context).await,
                _ => {
                    if let Some(mut superstate) = self.superstate() {
                        superstate.enter(shared_storage, context, levels - 1).await;
                    }
                    self.call_entry_action(shared_storage, context).await;
                }
            }
        };
        Box::pin(future)
    }

    /// Starting from the current state, climb a given amount of levels and execute all the
    /// the exit actions while going up to a certain superstate.
    fn exit<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        context: &'fut mut M::Context<'_>,
        levels: usize,
    ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
        let future = async move {
            match levels {
                0 => (),
                1 => self.call_exit_action(shared_storage, context).await,
                _ => {
                    self.call_exit_action(shared_storage, context).await;
                    if let Some(mut superstate) = self.superstate() {
                        superstate.exit(shared_storage, context, levels - 1).await;
                    }
                }
            }
        };
        Box::pin(future)
    }
}

impl<T, M> StateExt<M> for T
where
    Self: State<M> + Send,
    M: IntoStateMachine<State = T> + Send,
    for<'evt> M::Event<'evt>: Send + Sync,
    for<'ctx> M::Context<'ctx>: Send + Sync,
    for<'b> M::Superstate<'b>: Superstate<M> + Send,
{
}
