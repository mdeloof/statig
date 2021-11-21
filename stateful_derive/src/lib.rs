extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::visit::{self, Visit};
use syn::{
    ExprPath, Ident,
    Meta::NameValue,
    NestedMeta::{Lit, Meta},
};

#[proc_macro_attribute]
pub fn state_machine(_: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the token stream.
    let ast: syn::Item = syn::parse(input).unwrap();

    // Visit item impl
    let mut state_machine_visitor = StateMachineVisitor::default();
    state_machine_visitor.visit_item(&ast);

    // Get the state machine data.
    let item_impl = state_machine_visitor.item_impl.expect("no impl item found");
    let state_machine = StateMachine::from_ast(item_impl);

    let object_ident = state_machine.object_ident;
    let event_ident = get_event_ident(
        state_machine_visitor
            .state_handlers
            .first()
            .expect("no states found"),
    );

    let state_ident = state_machine.state_ident;
    let superstate_ident = state_machine.superstate_ident;

    let states: Vec<State> = state_machine_visitor
        .state_handlers
        .iter()
        .map(|s| State::from_ast(s))
        .collect();

    // Generate the idents of the enum variants.
    let state_variant_idents = states.iter().map(|s| &s.ident);

    // Generate the state handler matches.
    let state_to_handler_matches = states.iter().map(|s| {
        let state_variant_ident = &s.ident;
        let state_handler_ident = &s.state_handler;
        quote! { #state_ident::#state_variant_ident => #object_ident::#state_handler_ident }
    });

    // Generate the superstate state matches.
    let state_to_superstate_matches = states.iter().map(|s| {
        let state_variant_ident = &s.ident;
        match &s.superstate {
            Some(superstate_variant_ident) => {
                quote! { #state_ident::#state_variant_ident => Some(#superstate_ident::#superstate_variant_ident) }
            }
            None => quote! { #state_ident::#state_variant_ident => None },
        }
    });

    // Generate the entry action matches.
    let state_to_entry_action_matches = states.iter().map(|s| {
        let state_enum_variant = &s.ident;
        match &s.on_enter_handler {
            Some(on_enter_handler) => {
                quote! { #state_ident::#state_enum_variant => Some(#on_enter_handler) }
            }
            None => quote! { #state_ident::#state_enum_variant => None },
        }
    });

    // Generate the exit action matches.
    let state_to_exit_action_matches = states.iter().map(|s| {
        let state_variant_ident = &s.ident;
        match &s.on_exit_handler {
            Some(on_exit_handler) => {
                quote! { #state_ident::#state_variant_ident => Some(#on_exit_handler) }
            }
            None => quote! { #state_ident::#state_variant_ident => None },
        }
    });

    let superstates: Vec<State> = state_machine_visitor
        .superstate_handlers
        .iter()
        .map(|s| State::from_ast(s))
        .collect();

    // Generate the idents of the enum variants.
    let superstate_variant_idents = superstates.iter().map(|s| &s.ident);

    // Generate the state handler matches.
    let mut superstate_to_handler_matches = superstates.iter().map(|s| {
        let superstate_variant_ident = &s.ident;
        let superstate_handler_ident = &s.state_handler;
        quote! { #superstate_ident::#superstate_variant_ident => #object_ident::#superstate_handler_ident }
    }).collect::<Vec<proc_macro2::TokenStream>>();

    superstate_to_handler_matches.push(quote! { _ => |_, _| { stateful::Response::Handled } });

    // Generate the superstate state matches.
    let mut superstate_to_superstate_matches = superstates.iter().map(|s| {
        let superstate_variant_ident = &s.ident;
        match &s.superstate {
            Some(superstate_ident) => {
                quote! { #superstate_ident::#superstate_variant_ident => Some(#superstate_ident::#superstate_ident) }
            }
            None => quote! { #superstate_ident::#superstate_variant_ident => None },
        }
    }).collect::<Vec<proc_macro2::TokenStream>>();

    superstate_to_superstate_matches.push(quote! { _ => None });

    // Generate the on enter handler matches.
    let mut superstate_to_entry_action_matches = superstates.iter().map(|s| {
        let superstate_variant_ident = &s.ident;
        match &s.on_enter_handler {
            Some(on_enter_handler) => {
                quote! { #superstate_ident::#superstate_variant_ident => Some(#on_enter_handler) }
            }
            None => quote! { #superstate_ident::#superstate_variant_ident => None },
        }
    }).collect::<Vec<proc_macro2::TokenStream>>();

    superstate_to_entry_action_matches.push(quote! { _ => None });

    // Generate the on exit handler matches.
    let mut superstate_to_exit_action_matches = superstates.iter().map(|s| {
        let superstate_variant_ident = &s.ident;
        match &s.on_exit_handler {
            Some(on_exit_handler) => {
                quote! { #superstate_ident::#superstate_variant_ident => Some(#on_exit_handler) }
            }
            None => quote! { #superstate_ident::#superstate_variant_ident => None },
        }
    }).collect::<Vec<proc_macro2::TokenStream>>();

    superstate_to_exit_action_matches.push(quote! { _ => None });

    let gen = quote! {

        use stateful::state;
        use stateful::superstate;

        #ast

        #[derive(Copy, Clone, PartialEq, Debug)]
        pub enum #state_ident {
            #(#state_variant_idents),*
        }

        impl stateful::State for #state_ident {
            type Object = #object_ident;
            type Event = #event_ident;
            type Superstate = #superstate_ident;

            fn handler(&self) -> stateful::Handler<Self::Object, Self::Event> {
                match self {
                    #(#state_to_handler_matches),*
                }
            }

            fn superstate(&self) -> Option<Self::Superstate> {
                match self {
                    #(#state_to_superstate_matches),*
                }
            }

            fn entry_action(&self) -> Option<stateful::Action<Self::Object>> {
                match self {
                    #(#state_to_entry_action_matches),*
                }
            }

            fn exit_action(&self) -> Option<stateful::Action<Self::Object>> {
                match self {
                    #(#state_to_exit_action_matches),*
                }
            }
        }

        #[derive(Copy, Clone, PartialEq, Debug)]
        pub enum #superstate_ident {
            #(#superstate_variant_idents),*
        }

        impl stateful::Superstate for #superstate_ident {
            type Object = #object_ident;
            type Event = #event_ident;

            fn handler(&self) -> stateful::Handler<Self::Object, Self::Event> {
                match self {
                    #(#superstate_to_handler_matches),*
                }
            }

            fn superstate(&self) -> Option<Self> {
                match self {
                    #(#superstate_to_superstate_matches),*
                }
            }

            fn entry_action(&self) -> Option<stateful::Action<Self::Object>> {
                match self {
                    #(#superstate_to_entry_action_matches),*

                }
            }

            fn exit_action(&self) -> Option<stateful::Action<Self::Object>> {
                match self {
                    #(#superstate_to_exit_action_matches),*
                }
            }
        }

    };

    // Return the generated impl
    gen.into()
}

#[proc_macro_attribute]
pub fn state(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn superstate(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[derive(Default, Debug)]
struct StateMachineVisitor<'ast> {
    item_impl: Option<&'ast syn::ItemImpl>,
    state_handlers: Vec<&'ast syn::ImplItemMethod>,
    superstate_handlers: Vec<&'ast syn::ImplItemMethod>,
}

impl<'ast> StateMachineVisitor<'ast> {
    fn is_handler_sig(&self, sig: &syn::Signature) -> bool {
        use syn::{
            FnArg::{Receiver, Typed},
            ReturnType::Type,
        };
        match (&sig.inputs.iter().collect::<Vec<_>>()[..], &sig.output) {
            ([Receiver(_), Typed(_)], Type(.., _return_type)) => {
                if let Some(expected_sig) = self.state_handlers.first().map(|s| &s.sig) {
                    sig.inputs == expected_sig.inputs && sig.output == sig.output
                } else {
                    true
                }
            }
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for StateMachineVisitor<'ast> {
    fn visit_impl_item_method(&mut self, i: &'ast syn::ImplItemMethod) {
        for attr in i.attrs.iter() {
            match &attr.path {
                path if path.is_ident("state") => {
                    if self.is_handler_sig(&i.sig) {
                        self.state_handlers.push(i);
                    }
                }
                path if path.is_ident("superstate") => {
                    if self.is_handler_sig(&i.sig) {
                        self.superstate_handlers.push(i);
                    }
                }
                _ => {}
            }
        }
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        self.item_impl = Some(i);
        visit::visit_item_impl(self, i);
    }
}

#[derive(Debug)]
struct StateMachine {
    object_ident: Ident,
    state_ident: Ident,
    superstate_ident: Ident,
}

impl StateMachine {
    fn from_ast(impl_item: &syn::ItemImpl) -> Self {
        let object_ident = match &*impl_item.self_ty {
            syn::Type::Path(type_path) => type_path
                .path
                .segments
                .last()
                .expect("not found")
                .ident
                .clone(),
            _ => panic!("should have type"),
        };

        let mut state_ident = format_ident!("State");
        let mut superstate_ident = format_ident!("Superstate");

        let meta_items =
            match parse_state_handler_attribute(&impl_item.attrs, format_ident!("state")) {
                Some(meta_items) => meta_items,
                None => Vec::new(),
            };

        for meta_item in meta_items {
            match meta_item {
                Meta(NameValue(name_value)) if name_value.path.is_ident("name") => {
                    if let syn::Lit::Str(name_lit) = name_value.lit {
                        state_ident = format_ident!("{}", name_lit.value())
                    }
                }

                Lit(_) => panic!("unexpected literal"),

                test => panic!("unexpected {:?}", test),
            }
        }

        let meta_items =
            match parse_state_handler_attribute(&impl_item.attrs, format_ident!("superstate")) {
                Some(meta_items) => meta_items,
                None => Vec::new(),
            };

        for meta_item in meta_items {
            match meta_item {
                Meta(NameValue(name_value)) if name_value.path.is_ident("name") => {
                    if let syn::Lit::Str(name_lit) = name_value.lit {
                        superstate_ident = format_ident!("{}", name_lit.value())
                    }
                }

                Lit(_) => panic!("unexpected literal"),

                test => panic!("unexpected {:?}", test),
            }
        }

        Self {
            object_ident,
            state_ident,
            superstate_ident,
        }
    }
}

#[derive(Debug)]
struct State {
    ident: Ident,
    state_handler: Ident,
    superstate: Option<Ident>,
    on_enter_handler: Option<ExprPath>,
    on_exit_handler: Option<ExprPath>,
}

impl State {
    fn from_ast(method: &syn::ImplItemMethod) -> Self {
        let mut ident = format_ident!(
            "{}",
            snake_case_to_pascal_case(&method.sig.ident.to_string())
        );
        let state_handler = method.sig.ident.clone();
        let mut superstate = None;
        let mut on_enter_handler = None;
        let mut on_exit_handler = None;

        let meta_items = match parse_state_handler_attribute(&method.attrs, format_ident!("state"))
        {
            Some(meta_items) => meta_items,
            None => Vec::new(),
        };

        for meta_item in meta_items {
            match meta_item {
                Meta(NameValue(name_value)) if name_value.path.is_ident("name") => {
                    if let syn::Lit::Str(name_lit) = name_value.lit {
                        ident = format_ident!("{}", name_lit.value())
                    }
                }

                Meta(NameValue(name_value)) if name_value.path.is_ident("superstate") => {
                    if let syn::Lit::Str(superstate_lit) = name_value.lit {
                        superstate = Some(format_ident!("{}", superstate_lit.value()))
                    }
                }

                Meta(NameValue(name_value)) if name_value.path.is_ident("entry_action") => {
                    if let syn::Lit::Str(on_enter_lit) = name_value.lit {
                        on_enter_handler = Some(
                            syn::parse_str::<syn::ExprPath>(&on_enter_lit.value())
                                .expect("not a an expression"),
                        )
                    }
                }

                Meta(NameValue(name_value)) if name_value.path.is_ident("exit_action") => {
                    if let syn::Lit::Str(on_exit_lit) = name_value.lit {
                        on_exit_handler = Some(
                            syn::parse_str::<syn::ExprPath>(&on_exit_lit.value())
                                .expect("not an expression"),
                        )
                    }
                }

                Lit(_) => panic!("unexpected literal"),

                a => panic!("unexpected {:?}", a),
            }
        }

        Self {
            ident,
            state_handler,
            superstate,
            on_enter_handler,
            on_exit_handler,
        }
    }
}

fn parse_state_handler_attribute(
    attrs: &Vec<syn::Attribute>,
    attr_ident: Ident,
) -> Option<Vec<syn::NestedMeta>> {
    let state_attr = attrs.iter().find(|attr| attr.path.is_ident(&attr_ident));
    let state_attr = match state_attr {
        Some(attr) => attr,
        None => return None,
    };

    match state_attr.parse_meta() {
        Ok(syn::Meta::List(meta_items)) => Some(meta_items.nested.into_iter().collect()),
        Ok(syn::Meta::Path(_)) => Some(Vec::new()),
        Ok(_) => panic!("state attribute must be a list"),
        Err(_) => panic!("state attribute must follow meta syntax"),
    }
}

fn snake_case_to_pascal_case(snake: &str) -> String {
    let mut pascal = String::new();
    for part in snake.split("_") {
        let mut characters = part.chars();
        pascal.extend(
            characters
                .next()
                .map_or_else(String::new, |c| {
                    c.to_uppercase().chain(characters).collect()
                })
                .chars(),
        );
    }
    pascal
}

fn get_event_ident(ast: &syn::ImplItemMethod) -> &syn::Ident {
    let signature = &ast.sig;
    let second_input = signature.inputs.iter().nth(1);

    let event_type = if let Some(syn::FnArg::Typed(event_type)) = second_input {
        event_type
    } else {
        panic!("event input must be typed");
    };

    let event_type = if let syn::Type::Reference(event_type) = event_type.ty.as_ref() {
        event_type
    } else {
        panic!("event input must be a reference");
    };

    if let syn::Type::Path(event_type) = event_type.elem.as_ref() {
        &event_type.path.get_ident().unwrap()
    } else {
        panic!("event does not match");
    }
}
