use proc_macro2::TokenStream;
use proc_macro_error::abort;
use syn::{parse::Parser, punctuated::Punctuated, AttributeArgs, Item, ItemImpl, NestedMeta};

pub fn parse_args(args: TokenStream) -> AttributeArgs {
    let result: Result<Punctuated<NestedMeta, _>, _> =
        Punctuated::<syn::NestedMeta, syn::Token![,]>::parse_terminated.parse2(args);
    match result {
        Ok(args) => args.into_iter().collect(),
        Err(_) => unreachable!(),
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
        Err(_) => unreachable!(),
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
