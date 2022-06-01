use proc_macro2::TokenStream;
use proc_macro_error::abort;
use syn::{Item, ItemImpl};

pub fn parse(item: TokenStream) -> ItemImpl {
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

    parse(quote!(
        #[state_machine]
        impl Blinky {
            fn on(&mut self, event: &Event) -> Reponse<State> {
                Response::Handled
            }

            fn off(&mut self, event: &Event) -> Response<State> {
                Response::Handled
            }
        }
    ));
}

#[test]
#[should_panic]
fn invalid_input() {
    use quote::quote;

    parse(quote!(
        #[state_machine]
        struct Blinky {
            led: bool,
        }
    ));
}
