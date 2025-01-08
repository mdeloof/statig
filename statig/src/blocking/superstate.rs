use core::cmp::Ordering;

use crate::blocking::IntoStateMachine;
use crate::{Response, StateOrSuperstate};

/// An enum that represents the superstates of the state machine.
pub trait Superstate<M>
where
    M: IntoStateMachine,
{
    /// Call the handler for the current superstate.
    fn call_handler(
        &mut self,
        shared_storage: &mut M,
        event: &M::Event<'_>,
        context: &mut M::Context<'_>,
    ) -> Response<M::State>;

    #[allow(unused)]
    /// Call the entry action for the current superstate.
    fn call_entry_action(&mut self, shared_storage: &mut M, context: &mut M::Context<'_>) {}

    #[allow(unused)]
    /// Call the exit action for the current superstate.
    fn call_exit_action(&mut self, shared_storage: &mut M, context: &mut M::Context<'_>) {}

    /// Return the superstate of the current superstate, if there is one.
    fn superstate(&mut self) -> Option<M::Superstate<'_>>
    where
        Self: Sized,
    {
        None
    }
}

/// Extensions for `Superstate` trait.
pub trait SuperstateExt<'a, M>: Superstate<M>
where
    M: IntoStateMachine,
    Self: Sized,
    M::State: 'a,
    for<'b> M::Superstate<'b>: Superstate<M>,
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
    fn handle(
        &mut self,
        shared_storage: &mut M,
        event: &M::Event<'_>,
        context: &mut M::Context<'_>,
    ) -> Response<M::State>
    where
        Self: Sized,
    {
        let response = self.call_handler(shared_storage, event, context);

        match response {
            Response::Handled => Response::Handled,
            Response::Super => match self.superstate() {
                Some(mut superstate) => {
                    M::before_dispatch(
                        shared_storage,
                        StateOrSuperstate::Superstate(&superstate),
                        event,
                    );

                    let response = superstate.handle(shared_storage, event, context);

                    M::after_dispatch(
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
    }

    /// Starting from the current superstate, climb a given amount of levels and execute all the
    /// entry actions while going back down to the current superstate.
    fn enter(&mut self, shared_storage: &mut M, context: &mut M::Context<'_>, mut levels: usize) {
        match levels {
            0 => (),
            1 => self.call_entry_action(shared_storage, context),
            _ => {
                if let Some(mut superstate) = self.superstate() {
                    levels -= 1;
                    superstate.enter(shared_storage, context, levels);
                }
                self.call_entry_action(shared_storage, context);
            }
        }
    }

    /// Starting from the current superstate, climb a given amount of levels and execute all the
    /// the exit actions while going up to a certain superstate.
    fn exit(&mut self, shared_storage: &mut M, context: &mut M::Context<'_>, mut levels: usize) {
        match levels {
            0 => (),
            1 => self.call_exit_action(shared_storage, context),
            _ => {
                self.call_exit_action(shared_storage, context);
                if let Some(mut superstate) = self.superstate() {
                    levels -= 1;
                    superstate.exit(shared_storage, context, levels);
                }
            }
        }
    }
}

/// When no superstates are required, the user can pass the [`()`](unit) type.
impl<M> Superstate<M> for ()
where
    M: IntoStateMachine,
{
    fn call_handler(
        &mut self,
        _: &mut M,
        _: &M::Event<'_>,
        _: &mut M::Context<'_>,
    ) -> Response<M::State> {
        Response::Handled
    }

    fn call_entry_action(&mut self, _: &mut M, _: &mut M::Context<'_>) {}

    fn call_exit_action(&mut self, _: &mut M, _: &mut M::Context<'_>) {}

    fn superstate(&mut self) -> Option<M::Superstate<'_>>
    where
        Self: Sized,
    {
        None
    }
}

impl<'a, T, M> SuperstateExt<'a, M> for T
where
    T: Superstate<M>,
    M: IntoStateMachine,
    M::State: 'a,
    for<'b> M::Superstate<'b>: Superstate<M>,
{
}
