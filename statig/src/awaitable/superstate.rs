use core::cmp::Ordering;
use core::future::Future;
use core::pin::Pin;

use crate::IntoStateMachine;
use crate::Response;
use crate::StateOrSuperstate;

/// An enum that represents the superstates of the state machine.
pub trait Superstate<M>
where
    M: IntoStateMachine,
{
    /// Call the handler for the current superstate.
    fn call_handler<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        event: &'fut M::Event<'_>,
        context: &'fut mut M::Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Response<M::State>> + 'fut + Send>>;

    #[allow(unused)]
    /// Call the entry action for the current superstate.
    fn call_entry_action<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        context: &'fut mut M::Context<'_>,
    ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
        Box::pin(core::future::ready(()))
    }

    #[allow(unused)]
    /// Call the exit action for the current superstate.
    fn call_exit_action<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        context: &'fut mut M::Context<'_>,
    ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
        Box::pin(core::future::ready(()))
    }

    /// Return the superstate of the current superstate, if there is one.
    fn superstate(&mut self) -> Option<M::Superstate<'_>>
    where
        Self: Sized,
    {
        None
    }
}

/// Extensions for `Superstate` trait.
pub trait SuperstateExt<M>: Superstate<M>
where
    Self: Sized + Send,
    M: IntoStateMachine + Send,
    for<'evt> M::Event<'evt>: Send + Sync,
    for<'ctx> M::Context<'ctx>: Send + Sync,
    M::State: Send,
    for<'sub> M::Superstate<'sub>: Superstate<M> + Send,
{
    fn same_state(lhs: &M::Superstate<'_>, rhs: &M::Superstate<'_>) -> bool {
        use core::mem::{discriminant, transmute, Discriminant};

        // Generic associated types are invariant over any lifetime arguments, so the
        // compiler won't allow us to compare them directly. Instead we need to coerce them
        // to have the same lifetime by transmuting them to the same type.

        let lhs: Discriminant<M::Superstate<'_>> = unsafe { transmute(discriminant(lhs)) };
        let rhs: Discriminant<M::Superstate<'_>> = unsafe { transmute(discriminant(rhs)) };

        lhs == rhs
    }

    /// Get the depth of the current superstate.
    fn depth(&mut self) -> usize {
        match self.superstate() {
            Some(mut superstate) => superstate.depth() + 1,
            None => 1,
        }
    }

    /// Get the depth of the common ancestor of two states.
    fn common_ancestor_depth(
        mut source: M::Superstate<'_>,
        mut target: M::Superstate<'_>,
    ) -> usize {
        match source.depth().cmp(&target.depth()) {
            Ordering::Equal => match Self::same_state(&source, &target) {
                true => source.depth(),
                false => match (source.superstate(), target.superstate()) {
                    (Some(source), Some(target)) => Self::common_ancestor_depth(source, target),
                    _ => 0,
                },
            },
            Ordering::Greater => match source.superstate() {
                Some(superstate) => Self::common_ancestor_depth(superstate, target),
                None => 0,
            },
            Ordering::Less => match target.superstate() {
                Some(superstate) => Self::common_ancestor_depth(source, superstate),
                None => 0,
            },
        }
    }

    /// Handle the given event in the current superstate.
    fn handle<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        event: &'fut M::Event<'_>,
        context: &'fut mut M::Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Response<M::State>> + 'fut + Send>> {
        Box::pin(async move {
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

                        let response = superstate.handle(shared_storage, event, context).await;

                        M::AFTER_DISPATCH(
                            shared_storage,
                            StateOrSuperstate::Superstate(&superstate),
                            event,
                        );

                        response
                    }
                    None => Response::Super,
                },
                Response::Transition(state) => Response::Transition(state),
            }
        })
    }

    /// Starting from the current superstate, climb a given amount of levels and execute all the
    /// entry actions while going back down to the current superstate.
    fn enter<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        context: &'fut mut M::Context<'_>,
        mut levels: usize,
    ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
        Box::pin(async move {
            match levels {
                0 => (),
                1 => self.call_entry_action(shared_storage, context).await,
                _ => {
                    if let Some(mut superstate) = self.superstate() {
                        levels -= 1;
                        superstate.enter(shared_storage, context, levels).await;
                    }
                    self.call_entry_action(shared_storage, context).await;
                }
            }
        })
    }

    /// Starting from the current superstate, climb a given amount of levels and execute all the
    /// the exit actions while going up to a certain superstate.
    fn exit<'fut>(
        &'fut mut self,
        shared_storage: &'fut mut M,
        context: &'fut mut M::Context<'_>,
        mut levels: usize,
    ) -> Pin<Box<dyn Future<Output = ()> + 'fut + Send>> {
        Box::pin(async move {
            match levels {
                0 => (),
                1 => self.call_exit_action(shared_storage, context).await,
                _ => {
                    self.call_exit_action(shared_storage, context).await;
                    if let Some(mut superstate) = self.superstate() {
                        levels -= 1;
                        superstate.exit(shared_storage, context, levels).await;
                    }
                }
            }
        })
    }
}

/// When no superstates are required, the user can pass the [`()`](unit) type.
impl<M> Superstate<M> for ()
where
    M: IntoStateMachine + Send,
    M::State: Send,
    for<'evt> M::Event<'evt>: Send + Sync,
    for<'ctx> M::Context<'ctx>: Send + Sync,
{
    fn call_handler<'fut>(
        &'fut mut self,
        _: &'fut mut M,
        _: &'fut M::Event<'_>,
        _: &'fut mut M::Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Response<M::State>> + 'fut + Send>> {
        Box::pin(core::future::ready(Response::Handled))
    }

    fn call_entry_action(
        &mut self,
        _: &mut M,
        _: &mut M::Context<'_>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(core::future::ready(()))
    }

    fn call_exit_action(
        &mut self,
        _: &mut M,
        _: &mut M::Context<'_>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(core::future::ready(()))
    }

    fn superstate(&mut self) -> Option<M::Superstate<'_>>
    where
        Self: Sized,
    {
        None
    }
}

impl<T, M> SuperstateExt<M> for T
where
    Self: Superstate<M> + Send,
    M: IntoStateMachine + Send,
    for<'evt> M::Event<'evt>: Send + Sync,
    for<'ctx> M::Context<'ctx>: Send + Sync,
    M::State: Send,
    for<'sub> M::Superstate<'sub>: Superstate<M> + Send,
{
}
