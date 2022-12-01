use crate::Response;
use crate::StateMachine;
use crate::StateOrSuperstate;
use crate::SuperstateExt;

/// An enum that represents the leaf states of the state machine.
pub trait State<M>
where
    Self: Sized,
    M: StateMachine,
{
    /// Call the handler for the current state and let it handle the given event.
    fn call_handler(
        &mut self,
        context: &mut M,
        event: &<M as StateMachine>::Event<'_>,
    ) -> Response<Self>;

    /// Call the entry action for the current state.
    fn call_entry_action(&mut self, #[allow(unused)] context: &mut M) {}

    /// Call the exit action for the current state.
    fn call_exit_action(&mut self, #[allow(unused)] context: &mut M) {}

    /// Return the superstate of the current state, if there is one.
    fn superstate(&mut self) -> Option<<M as StateMachine>::Superstate<'_>> {
        None
    }
}

/// Extensions for `State` trait.
pub trait StateExt<M>: State<M>
where
    M: StateMachine<State = Self>,
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
            (Some(source), Some(target)) => {
                <<M as StateMachine>::Superstate<'_> as SuperstateExt<M>>::common_ancestor_depth(
                    source, target,
                )
            }
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
            let common_state_depth =
                <M as StateMachine>::Superstate::common_ancestor_depth(source, target);
            (
                source_depth - common_state_depth,
                target_depth - common_state_depth,
            )
        } else {
            (source_depth, target_depth)
        }
    }

    /// Handle the given event in the current state.
    fn handle(&mut self, context: &mut M, event: &<M as StateMachine>::Event<'_>) -> Response<Self>
    where
        Self: Sized,
    {
        M::ON_DISPATCH(context, StateOrSuperstate::State(self), event);

        let response = self.call_handler(context, event);

        match response {
            Response::Handled => Response::Handled,
            Response::Super => match self.superstate() {
                Some(mut superstate) => {
                    M::ON_DISPATCH(context, StateOrSuperstate::Superstate(&superstate), event);

                    superstate.handle(context, event)
                }
                None => Response::Super,
            },
            Response::Transition(state) => Response::Transition(state),
        }
    }

    /// Starting from the current state, climb a given amount of levels and execute all the
    /// entry actions while going back down to the current state.
    fn enter(&mut self, context: &mut M, levels: usize) {
        match levels {
            0 => (),
            1 => self.call_entry_action(context),
            _ => {
                if let Some(mut superstate) = self.superstate() {
                    superstate.enter(context, levels - 1);
                }
                self.call_entry_action(context);
            }
        }
    }

    /// Starting from the current state, climb a given amount of levels and execute all the
    /// the exit actions while going up to a certain superstate.
    fn exit(&mut self, context: &mut M, levels: usize) {
        match levels {
            0 => (),
            1 => self.call_exit_action(context),
            _ => {
                self.call_exit_action(context);
                if let Some(mut superstate) = self.superstate() {
                    superstate.exit(context, levels - 1);
                }
            }
        }
    }
}

impl<T, M> StateExt<M> for T
where
    T: State<M>,
    M: StateMachine<State = T>,
{
}
