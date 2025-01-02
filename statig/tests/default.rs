#[cfg(test)]
mod tests {
    use statig::blocking::*;

    pub struct Foo;

    #[state_machine(initial = "State::bar()")]
    impl Foo {
        #[state]
        fn bar(#[default] _local: &mut usize) -> Response<State> {
            Handled
        }
    }
}
