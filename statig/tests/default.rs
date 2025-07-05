#[cfg(test)]
mod tests {
    #![allow(unused)]

    use statig::blocking::*;

    pub struct Foo;

    #[state_machine(initial = State::bar())]
    impl Foo {
        #[state]
        fn bar(
            #[default] local: &mut usize,
            #[default = 100] local_2: &mut usize,
        ) -> Outcome<State> {
            Handled
        }
    }

    #[test]
    fn my_cool_test() {
        let foo = Foo.state_machine();
        match foo.state() {
            State::Bar { local, local_2 } => {
                assert_eq!(*local, 0);
                assert_eq!(*local_2, 100);
            }
        }
    }
}
