use statig::blocking::*;
use std::marker::PhantomData;

pub enum Event<T> {
    Foo,
    Bar(T),
}

// All generics must be declared on the shared storage type. They can then
// be used inside the state and action handlers.
#[derive(Default)]
pub struct Machine<'a, T, const SIZE: usize> {
    marker: PhantomData<&'a T>,
}

#[state_machine(initial = "State::foo()")]
impl<'d, T, const SIZE: usize> Machine<'d, T, SIZE>
where
    T: std::fmt::Debug + Clone + Default + Copy,
    T: 'static,
{
    #[state]
    fn foo(event: &Event<T>) -> Response<State<T, SIZE>> {
        match event {
            Event::Bar(value) => Transition(State::bar(*value, [T::default(); SIZE])),
            _ => Super,
        }
    }

    #[allow(unused)]
    #[action]
    fn enter_bar(value: &mut T) {
        dbg!(value);
    }

    #[allow(unused)]
    #[state(superstate = "barbar", entry_action = "enter_bar")]
    fn bar(value: &mut T, buffer: &[T; SIZE], event: &Event<T>) -> Response<State<T, SIZE>> {
        match event {
            Event::Foo => Transition(State::foo()),
            _ => Super,
        }
    }

    #[allow(unused)]
    #[superstate]
    fn barbar(value: &mut T) -> Response<State<T, SIZE>> {
        Super
    }
}

fn main() {
    let mut machine = Machine::<Option<&u32>, 45>::default()
        .uninitialized_state_machine()
        .init();
    machine.handle(&Event::Bar(None));
}
