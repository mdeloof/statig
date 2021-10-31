extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::visit::{self, Visit};
use syn::{
    Ident,
    Meta::NameValue,
    NestedMeta::{Lit, Meta},
};

#[proc_macro_attribute]
pub fn derive_state(_: TokenStream, input: TokenStream) -> TokenStream {
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
    let states: Vec<State> = state_machine_visitor
        .state_handlers
        .iter()
        .map(|s| State::from_ast(s))
        .collect();

    // Generate the idents of the enum variants.
    let state_variant_idents = states.iter().map(|s| &s.ident);

    // Generate the state handler matches.
    let state_to_state_handler_matches = states.iter().map(|s| {
        let state_variant_ident = &s.ident;
        let state_handler_ident = &s.state_handler;
        quote! { #state_ident::#state_variant_ident => #object_ident::#state_handler_ident }
    });

    // Generate the parent state matches.
    let state_to_parent_matches = states.iter().map(|s| {
        let state_variant_ident = &s.ident;
        match &s.parent {
            Some(parent_ident) => {
                quote! { #state_ident::#state_variant_ident => Some(#state_ident::#parent_ident) }
            }
            None => quote! { #state_ident::#state_variant_ident => None },
        }
    });

    // Generate the on enter handler matches.
    let state_to_on_enter_handler_matches = states
        .iter()
        .map(|s| {
            let state_enum_variant = &s.ident;
            match &s.on_enter_handler {
                Some(on_enter_handler) => quote! { #state_ident::#state_enum_variant => Some(#object_ident::#on_enter_handler) },
                None => quote! { #state_ident::#state_enum_variant => None }
            }
        });

    // Generate the on exit handler matches.
    let state_to_on_exit_handler_matches = states
        .iter()
        .map(|s| {
            let state_variant_ident = &s.ident;
            match &s.on_exit_handler {
                Some(on_exit_handler) => quote! { #state_ident::#state_variant_ident => Some(#object_ident::#on_exit_handler) },
                None => quote! { #state_ident::#state_variant_ident => None }
            }
        });

    let gen = quote! {

        use stateful::state;

        #ast

        #[derive(Copy, Clone, PartialEq)]
        enum #state_ident {
            #(#state_variant_idents),*
        }

        impl stateful::State for #state_ident {
            type Object = #object_ident;
            type Event = #event_ident;

            fn state_handler(&self) -> stateful::StateHandler<Self::Object, Self::Event> {
                match self {
                    #(#state_to_state_handler_matches),*
                }
            }

            fn parent_state(&self) -> Option<Self> {
                match self {
                    #(#state_to_parent_matches),*
                }
            }

            fn state_on_enter_handler(&self) -> Option<stateful::StateOnEnterHandler<Self::Object>> {
                match self {
                    #(#state_to_on_enter_handler_matches),*
                }
            }

            fn state_on_exit_handler(&self) -> Option<stateful::StateOnExitHandler<Self::Object>> {
                match self {
                    #(#state_to_on_exit_handler_matches),*
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

#[derive(Default, Debug)]
struct StateMachineVisitor<'ast> {
    item_impl: Option<&'ast syn::ItemImpl>,
    state_handlers: Vec<&'ast syn::ImplItemMethod>,
}

impl<'ast> Visit<'ast> for StateMachineVisitor<'ast> {
    fn visit_impl_item_method(&mut self, i: &'ast syn::ImplItemMethod) {
        match (&i.sig.inputs.iter().collect::<Vec<_>>()[..], &i.sig.output) {
            // Match the expected signature of a state handler function.
            ([syn::FnArg::Receiver(_), syn::FnArg::Typed(_)], syn::ReturnType::Type(.., _return_type)) => {
                if let Some(sig) = self.state_handlers.first().map(|s| &s.sig) {
                    assert_eq!(sig.inputs, i.sig.inputs, "all state handlers should have the same signature");
                    assert_eq!(sig.output, i.sig.output, "all state handlers should have the same signature");
                }
                self.state_handlers.push(i);
            }
            _ => panic!("state handlers should have a signature matching `fn(&mut Self, &Event) -> Response`"),
        }
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        self.item_impl = Some(i);
        visit::visit_item_impl(self, i);
    }
}

struct StateMachine {
    object_ident: Ident,
    state_ident: Ident,
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

        let mut state_ident = format_ident!("{}State", object_ident);

        let meta_items = parse_state_handler_attribute(&impl_item.attrs);

        for meta_item in meta_items {
            match meta_item {
                Meta(NameValue(name_value)) if name_value.path.is_ident("name") => {
                    if let syn::Lit::Str(name_lit) = name_value.lit {
                        state_ident = format_ident!("{}", name_lit.value())
                    }
                }

                Lit(_) => panic!("unnexpected literal"),

                _ => panic!("unnexpected"),
            }
        }

        Self {
            object_ident,
            state_ident,
        }
    }
}

struct State {
    ident: Ident,
    state_handler: Ident,
    parent: Option<Ident>,
    on_enter_handler: Option<Ident>,
    on_exit_handler: Option<Ident>,
}

impl State {
    fn from_ast(method: &syn::ImplItemMethod) -> Self {
        let ident = format_ident!(
            "{}",
            snake_case_to_pascal_case(&method.sig.ident.to_string())
        );
        let state_handler = method.sig.ident.clone();
        let mut parent = None;
        let mut on_enter_handler = None;
        let mut on_exit_handler = None;

        let meta_items = parse_state_handler_attribute(&method.attrs);

        for meta_item in meta_items {
            match meta_item {
                Meta(NameValue(name_value)) if name_value.path.is_ident("parent") => {
                    if let syn::Lit::Str(parent_lit) = name_value.lit {
                        parent = Some(format_ident!("{}", parent_lit.value()))
                    }
                }

                Meta(NameValue(name_value)) if name_value.path.is_ident("on_enter") => {
                    if let syn::Lit::Str(on_enter_lit) = name_value.lit {
                        on_enter_handler = Some(format_ident!("{}", on_enter_lit.value()))
                    }
                }

                Meta(NameValue(name_value)) if name_value.path.is_ident("on_exit") => {
                    if let syn::Lit::Str(on_exit_lit) = name_value.lit {
                        on_exit_handler = Some(format_ident!("{}", on_exit_lit.value()))
                    }
                }

                Lit(_) => panic!("unnexpected literal"),

                _ => panic!("unnexpected"),
            }
        }

        Self {
            ident,
            state_handler,
            parent,
            on_enter_handler,
            on_exit_handler,
        }
    }
}

fn parse_state_handler_attribute(attrs: &Vec<syn::Attribute>) -> Vec<syn::NestedMeta> {
    let state_attr = attrs.iter().find(|attr| attr.path.is_ident("state"));
    let state_attr = match state_attr {
        Some(attr) => attr,
        None => return Vec::new(),
    };

    match state_attr.parse_meta() {
        Ok(syn::Meta::List(meta_items)) => meta_items.nested.into_iter().collect(),
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
