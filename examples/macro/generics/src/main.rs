#![allow(unused)]

use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

use statig::prelude::*;

pub enum Event<T> {
    Foo,
    Bar(T),
}

// All generics must be declared on the shared storage type. They can then
// be used inside the state and action handlers.
#[derive(Default)]
pub struct Machine<'a, T, A, const SIZE: usize> {
    marker: PhantomData<(&'a T, A)>,
}

use statig::prelude::*;

#[state_machine(initial = "State::foo()")]
impl<'d, T, A, const SIZE: usize> Machine<'d, T, A, SIZE>
where
    // Note that we need to introduce a `'static` trait bound on the generics. This
    // constrains the generic parameters to types that can be owned by the state machine.
    T: 'static + Debug + Clone + Copy + Default,
    A: 'static + Deref,
{
    #[state]
    fn foo(event: &Event<T>) -> Response<State<T, SIZE>> {
        match event {
            Event::Bar(value) => Transition(State::bar(*value, [T::default(); SIZE])),
            _ => Super,
        }
    }

    #[action]
    fn enter_bar(value: &mut T) {
        println!("{:?}", value);
    }

    #[state(superstate = "foo_and_bar", entry_action = "enter_bar")]
    fn bar(value: &mut T, buffer: &[T; SIZE], event: &Event<T>) -> Response<State<T, SIZE>> {
        match event {
            Event::Foo => Transition(State::foo()),
            _ => Super,
        }
    }

    #[superstate]
    fn foo_and_bar(value: &mut T) -> Response<State<T, SIZE>> {
        Super
    }
}

fn main() {
    let mut machine = Machine::<Option<&u32>, Box<u32>, 45>::default()
        .uninitialized_state_machine()
        .init();
    machine.handle(&Event::Bar(None));
}
