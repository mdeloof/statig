use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use syn::{parse::Parser, punctuated::Punctuated, Item, ItemImpl, Meta, Token};

pub fn parse_args(args: TokenStream) -> Vec<Meta> {
    let result = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(args);
    match result {
        Ok(args) => args.into_iter().collect(),
        Err(error) => abort!(error),
    }
}

pub fn parse_input(item: TokenStream) -> ItemImpl {
    let result: Result<Item, _> = syn::parse2(item);
    match result {
        Ok(Item::Impl(item_impl)) => item_impl,
        Ok(item) => abort!(
            item,
            "expected impl block";
            help = "`state_machine` can only be used on a impl block"
        ),
        Err(error) => abort!(error),
    }
}

#[test]
fn valid_input() {
    use quote::quote;

    let token_stream = quote!(
        #[state_machine]
        impl Blinky {
            fn on(&mut self, event: &Event) -> Reponse<State> {
                Response::Handled
            }

            fn off(&mut self, event: &Event) -> Response<State> {
                Response::Handled
            }
        }
    );

    parse_input(token_stream);
}

#[test]
#[should_panic]
fn invalid_input() {
    use quote::quote;

    let token_stream = quote!(
        #[state_machine]
        struct Blinky {
            led: bool,
        }
    );

    parse_input(token_stream);
}

#[test]
fn valid_args() {
    use quote::quote;

    let token_stream = quote!(initial = "State::foo()");

    parse_args(token_stream);
}
