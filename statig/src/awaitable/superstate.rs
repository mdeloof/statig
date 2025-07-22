use core::cmp::Ordering;
use core::future::Future;

use crate::awaitable::IntoStateMachine;
use crate::Outcome;

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
    ) -> impl Future<Output = Outcome<M::State, M::Response>>;

    #[allow(unused)]
    /// Call the entry action for the current superstate.
    fn call_entry_action(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'_>,
    ) -> impl Future<Output = ()> {
        core::future::ready(())
    }

    #[allow(unused)]
    /// Call the exit action for the current superstate.
    fn call_exit_action(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'_>,
    ) -> impl Future<Output = ()> {
        core::future::ready(())
    }

    /// Return the superstate of the current superstate, if there is one.
    fn superstate(&mut self) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

/// Extensions for `Superstate` trait.
pub trait SuperstateExt<M>: Superstate<M>
where
    Self: Sized,
    M: IntoStateMachine,
    for<'sub> M::Superstate<'sub>: Superstate<M>,
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
    ) -> impl Future<Output = Outcome<M::State, M::Response>> {
        core::future::ready(Outcome::Handled(M::default_response()))
    }

    fn call_entry_action(&mut self, _: &mut M, _: &mut M::Context<'_>) -> impl Future<Output = ()> {
        core::future::ready(())
    }

    fn call_exit_action(&mut self, _: &mut M, _: &mut M::Context<'_>) -> impl Future<Output = ()> {
        core::future::ready(())
    }

    fn superstate(&mut self) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

impl<T, M> SuperstateExt<M> for T
where
    Self: Superstate<M>,
    M: IntoStateMachine,
    for<'sub> M::Superstate<'sub>: Superstate<M>,
{
}
