#[cfg(test)]
mod tests {
    use statig::blocking::*;

    pub struct Foo;

    #[state_machine(initial = "State::bar()")]
    impl Foo {
        #[state]
        fn bar(
            #[default] _local: &mut usize,
            #[default = "100"] _local_2: &mut usize,
        ) -> Response<State> {
            Handled
        }
    }
}
