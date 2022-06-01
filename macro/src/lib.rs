extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod analyze;
mod codegen;
mod lower;
mod parse;

use analyze::analyze;
use codegen::codegen;
use lower::lower;
use parse::parse;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn state_machine(_: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl = parse(input.into());
    let model = analyze(item_impl);
    let ir = lower(model);
    let rust = codegen(ir);
    rust.into()
}

#[proc_macro_attribute]
pub fn state(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn superstate(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn action(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}
