#[cfg(test)]
#[allow(unused)]
mod tests {
    use std::marker::PhantomData;

    use statig::prelude::*;

    #[derive(Default)]
    struct Counter<'a, T, const SIZE: usize> {
        marker: PhantomData<[&'a T; SIZE]>,
    }

    #[derive(Clone)]
    struct ExternalContext(usize);

    enum Event {}

    #[state_machine(initial = "State::uninitialized()")]
    impl<'a, T, const SIZE: usize> Counter<'a, T, SIZE>
    where
        T: 'static + Default + Copy,
    {
        #[state]
        fn uninitialized() -> Response<State<T, SIZE>> {
            Handled
        }

        #[state]
        fn initialized(array: &mut [T; SIZE]) -> Response<State<T, SIZE>> {
            Handled
        }
    }
}
