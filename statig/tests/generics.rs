#[cfg(test)]
#[allow(unused)]
mod tests {
    use std::marker::PhantomData;
    use std::ops::Deref;

    use statig::prelude::*;

    #[derive(Default)]
    struct Counter<'a, T, A, B, const SIZE: usize> {
        marker: PhantomData<[&'a T; SIZE]>,
        marker2: PhantomData<(A, B)>,
    }

    #[derive(Clone)]
    struct ExternalContext(usize);

    enum Event {}

    #[state_machine(initial = State::a())]
    impl<'a, T, A, B, const SIZE: usize> Counter<'a, T, A, B, SIZE>
    where
        T: 'static + Default + Copy,
        A: 'static + Deref<Target = B>,
        B: 'static,
    {
        #[state]
        fn a() -> Outcome<State<T, B, SIZE>> {
            Handled
        }

        #[state]
        fn b(array: &mut [T; SIZE]) -> Outcome<State<T, B, SIZE>> {
            Handled
        }

        #[state]
        fn c(deref: &B) -> Outcome<State<T, B, SIZE>> {
            Handled
        }
    }
}
