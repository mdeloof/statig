use core::cmp::Ordering;

use crate::Response;
use crate::StateMachine;
use crate::StateOrSuperstate;

/// An enum that represents the superstates of the state machine.
pub trait Superstate<M>
where
    M: StateMachine,
{
    /// Call the handler for the current superstate.
    fn call_handler(
        &mut self,
        shared_storage: &mut M,
        event: &<M as StateMachine>::Event<'_>,
    ) -> Response<<M as StateMachine>::State>;

    /// Call the entry action for the current superstate.
    fn call_entry_action(&mut self, _object: &mut M) {}

    /// Call the exit action for the current superstate.
    fn call_exit_action(&mut self, _object: &mut M) {}

    /// Return the superstate of the current superstate, if there is one.
    fn superstate(&mut self) -> Option<<M as StateMachine>::Superstate<'_>>
    where
        Self: Sized,
    {
        None
    }
}

/// Extensions for `Superstate` trait.
pub trait SuperstateExt<M>: Superstate<M>
where
    M: StateMachine,
    Self: Sized,
{
    fn same_state(
        lhs: &<M as StateMachine>::Superstate<'_>,
        rhs: &<M as StateMachine>::Superstate<'_>,
    ) -> bool {
        use core::mem::{discriminant, transmute_copy, Discriminant};

        // Generic associated types are invariant over any lifetime arguments, so the
        // compiler won't allow us to compare them directly. Instead we need to coerce them
        // to have the same lifetime by transmuting them to the same type.

        let lhs: Discriminant<<M as StateMachine>::Superstate<'_>> =
            unsafe { transmute_copy(&discriminant(lhs)) };
        let rhs: Discriminant<<M as StateMachine>::Superstate<'_>> =
            unsafe { transmute_copy(&discriminant(rhs)) };

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
        mut source: <M as StateMachine>::Superstate<'_>,
        mut target: <M as StateMachine>::Superstate<'_>,
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
        event: &<M as StateMachine>::Event<'_>,
    ) -> Response<<M as StateMachine>::State>
    where
        Self: Sized,
    {
        let response = self.call_handler(shared_storage, event);

        match response {
            Response::Handled => Response::Handled,
            Response::Super => match self.superstate() {
                Some(mut superstate) => {
                    M::ON_DISPATCH(
                        shared_storage,
                        StateOrSuperstate::Superstate(&superstate),
                        event,
                    );

                    superstate.handle(shared_storage, event)
                }
                None => Response::Super,
            },
            Response::Transition(state) => Response::Transition(state),
        }
    }

    /// Starting from the current superstate, climb a given amount of levels and execute all the
    /// entry actions while going back down to the current superstate.
    fn enter(&mut self, shared_storage: &mut M, mut levels: usize) {
        match levels {
            0 => (),
            1 => self.call_entry_action(shared_storage),
            _ => {
                if let Some(mut superstate) = self.superstate() {
                    levels -= 1;
                    superstate.enter(shared_storage, levels);
                }
                self.call_entry_action(shared_storage);
            }
        }
    }

    /// Starting from the current superstate, climb a given amount of levels and execute all the
    /// the exit actions while going up to a certain superstate.
    fn exit(&mut self, shared_storage: &mut M, mut levels: usize) {
        match levels {
            0 => (),
            1 => self.call_exit_action(shared_storage),
            _ => {
                self.call_exit_action(shared_storage);
                if let Some(mut superstate) = self.superstate() {
                    levels -= 1;
                    superstate.exit(shared_storage, levels);
                }
            }
        }
    }
}

/// When no superstates are required, the user can pass the [`()`](unit) type.
impl<M> Superstate<M> for ()
where
    M: StateMachine,
{
    fn call_handler(
        &mut self,
        _shared_storage: &mut M,
        _event: &<M as StateMachine>::Event<'_>,
    ) -> Response<<M as StateMachine>::State> {
        Response::Handled
    }

    fn call_entry_action(&mut self, _object: &mut M) {}

    fn call_exit_action(&mut self, _object: &mut M) {}

    fn superstate(&mut self) -> Option<<M as StateMachine>::Superstate<'_>>
    where
        Self: Sized,
    {
        None
    }
}

impl<T, M> SuperstateExt<M> for T
where
    T: Superstate<M>,
    M: StateMachine,
{
}
