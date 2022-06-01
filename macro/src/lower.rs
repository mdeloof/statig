use proc_macro2::Span;
use proc_macro_error::abort;
use quote::format_ident;
use std::collections::HashMap;
use syn::ItemFn;
use syn::{parse_quote, Expr, ItemImpl, Lifetime, Type};
use syn::{ExprCall, FnArg, Ident, Pat, Path, Variant};

use crate::analyze;
use crate::analyze::Model;

const SUPERSTATE_LIFETIME: &str = "'a";

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Ir {
    pub item_impl: ItemImpl,
    pub state_machine: StateMachine,
    pub states: HashMap<Ident, State>,
    pub superstates: HashMap<Ident, Superstate>,
}

pub struct StateMachine {
    pub object_ty: Type,
    pub state_ty: Type,
    pub state_derives: Vec<Path>,
    pub superstate_ident: Ident,
    pub superstate_ty: Type,
    pub superstate_derives: Vec<Path>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub variant: Variant,
    pub pat: Pat,
    pub pat_ignore: Pat,
    pub handler_call: ExprCall,
    pub entry_action_call: Expr,
    pub exit_action_call: Expr,
    pub superstate_pat: Pat,
    pub constructor: ItemFn,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Superstate {
    pub variant: Variant,
    pub pat: Pat,
    pub pat_ignore: Pat,
    pub handler_call: ExprCall,
    pub entry_action_call: Expr,
    pub exit_action_call: Expr,
    pub superstate_pat: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    pub handler_call: ExprCall,
}

pub fn lower(model: Model) -> Ir {
    let item_impl = model.item_impl;

    let state_name = &model.state_machine.state_name;
    let state_ty = parse_quote!(#state_name);

    let superstate_ident = &model.state_machine.superstate_name;
    let mut superstate_ty = parse_quote!(#superstate_ident);

    let mut states: HashMap<Ident, State> = model
        .states
        .iter()
        .map(|(key, value)| (key.clone(), lower_state(&value, &model.state_machine)))
        .collect();

    let mut superstates: HashMap<Ident, Superstate> = model
        .superstates
        .iter()
        .inspect(|(_, value)| {
            if !value.state_inputs.is_empty() {
                superstate_ty = parse_quote!(#superstate_ident <'a>);
            }
        })
        .map(|(key, value)| (key.clone(), lower_superstate(&value, &model.state_machine)))
        .collect();

    let actions: HashMap<Ident, Action> = model
        .actions
        .iter()
        .map(|(key, value)| (key.clone(), lower_action(&value, &model.state_machine)))
        .collect();

    // Linking states
    for (key, state) in &mut states {
        if let Some(superstate) = model
            .states
            .get(key)
            .and_then(|state| state.superstate.as_ref())
        {
            match superstates.get(&superstate) {
                Some(superstate) => {
                    let superstate_pat = &superstate.pat;
                    state.superstate_pat = parse_quote!(Some(#superstate_pat))
                }
                None => abort!(superstate, "superstate not found"),
            }
        }

        if let Some(entry_action) = model
            .states
            .get(key)
            .and_then(|state| state.entry_action.as_ref())
        {
            match actions.get(&entry_action) {
                Some(action) => state.entry_action_call = action.handler_call.clone().into(),
                None => abort!(entry_action, "entry action not found"),
            }
        }

        if let Some(exit_action) = model
            .states
            .get(key)
            .and_then(|state| state.exit_action.as_ref())
        {
            match actions.get(&exit_action) {
                Some(action) => state.exit_action_call = action.handler_call.clone().into(),
                None => abort!(exit_action, "exit action not found"),
            }
        }
    }

    let superstates_clone = superstates.clone();

    // Linking superstates
    for key in superstates_clone.keys() {
        if let Some(superstate_superstate) = model
            .superstates
            .get(key)
            .and_then(|state| state.superstate.as_ref())
        {
            match superstates_clone.get(&superstate_superstate) {
                Some(superstate_superstate) => {
                    let superstate_superstate_pat = &superstate_superstate.pat;
                    superstates.get_mut(key).map(|superstate| {
                        superstate.superstate_pat = parse_quote!(Some(#superstate_superstate_pat))
                    });
                }
                None => abort!(superstate_superstate, "superstate not found"),
            }
        }

        if let Some(entry_action) = model
            .superstates
            .get(key)
            .and_then(|state| state.entry_action.as_ref())
        {
            match actions.get(&entry_action) {
                Some(action) => {
                    superstates.get_mut(key).map(|superstate| {
                        superstate.entry_action_call = action.handler_call.clone().into();
                    });
                }
                None => abort!(entry_action, "action not found"),
            }
        }

        if let Some(exit_action) = model
            .superstates
            .get(key)
            .and_then(|state| state.exit_action.as_ref())
        {
            match actions.get(&exit_action) {
                Some(action) => {
                    superstates.get_mut(key).map(|superstate| {
                        superstate.exit_action_call = action.handler_call.clone().into();
                    });
                }
                None => abort!(exit_action, "action not found"),
            }
        }
    }

    let object_ty = model.state_machine.object_ty;
    let state_derives = model.state_machine.state_derives;

    let superstate_ident = model.state_machine.superstate_name;
    let superstate_derives = model.state_machine.superstate_derives;

    let state_machine = StateMachine {
        object_ty,
        state_ty,
        state_derives,
        superstate_ident,
        superstate_ty,
        superstate_derives,
    };

    Ir {
        state_machine,
        item_impl,
        states,
        superstates,
    }
}

pub fn lower_state(state: &analyze::State, state_machine: &analyze::StateMachine) -> State {
    let variant_name = snake_case_to_pascal_case(&state.handler_name);
    let state_handler_name = &state.handler_name;
    let object_ty = &state_machine.object_ty;
    let state_name = &state_machine.state_name;
    let pat_ignore = parse_quote!(#state_name::#variant_name { .. });

    let variant_fields: Vec<Expr> = state
        .state_inputs
        .iter()
        .map(fn_arg_to_state_field)
        .collect();
    let pat_fields: Vec<Ident> = state.state_inputs.iter().map(fn_arg_to_ident).collect();
    let handler_inputs: Vec<Ident> = state.inputs.iter().map(fn_arg_to_ident).collect();

    let variant = parse_quote!(#variant_name { #(#variant_fields),* });
    let pat = parse_quote!(#state_name::#variant_name { #(#pat_fields),*});
    let constructor = parse_quote!(const fn #state_handler_name ( #(#variant_fields),* ) -> Self { Self::#variant_name { #(#pat_fields),*} });
    let handler_call = parse_quote!(#object_ty::#state_handler_name(#(#handler_inputs),*));
    let entry_action_call = parse_quote!({});
    let exit_action_call = parse_quote!({});
    let superstate_pat = parse_quote!(None);

    State {
        variant,
        pat,
        constructor,
        pat_ignore,
        handler_call,
        entry_action_call,
        exit_action_call,
        superstate_pat,
    }
}

pub fn lower_superstate(
    superstate: &analyze::Superstate,
    state_machine: &analyze::StateMachine,
) -> Superstate {
    let superstate_name = snake_case_to_pascal_case(&superstate.handler_name);
    let superstate_handler_name = &superstate.handler_name;
    let object_ty = &state_machine.object_ty;
    let superstate_ty = &state_machine.superstate_name;
    let pat_ignore = parse_quote!(#superstate_ty::#superstate_name { .. });

    let variant_fields: Vec<Expr> = superstate
        .state_inputs
        .iter()
        .map(fn_arg_to_superstate_field)
        .collect();
    let pat_fields: Vec<Ident> = superstate
        .state_inputs
        .iter()
        .map(fn_arg_to_ident)
        .collect();
    let handler_inputs: Vec<Ident> = superstate.inputs.iter().map(fn_arg_to_ident).collect();

    let variant = parse_quote!(#superstate_name { #(#variant_fields),* });
    let pat = parse_quote!(#superstate_ty::#superstate_name { #(#pat_fields),*});
    let handler_call = parse_quote!(#object_ty::#superstate_handler_name(#(#handler_inputs),*));
    let entry_action_call = parse_quote!({});
    let exit_action_call = parse_quote!({});
    let superstate_pat = parse_quote!(None);

    Superstate {
        variant,
        pat,
        pat_ignore,
        handler_call,
        entry_action_call,
        exit_action_call,
        superstate_pat,
    }
}

pub fn lower_action(action: &analyze::Action, state_machine: &analyze::StateMachine) -> Action {
    let action_handler_name = &action.handler_name;
    let object_ty = &state_machine.object_ty;

    let mut call_inputs: Vec<Ident> = Vec::new();

    for input in &action.inputs {
        match input {
            FnArg::Receiver(_) => {
                call_inputs.insert(0, parse_quote!(object));
            }

            // Typed argument.
            FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                Pat::Ident(pat_ident) => {
                    let field_ident = &pat_ident.ident;
                    call_inputs.push(parse_quote!(#field_ident));
                }
                _ => todo!(),
            },
        }
    }

    let handler_call = parse_quote!(#object_ty::#action_handler_name(#(#call_inputs),*));

    Action { handler_call }
}

fn fn_arg_to_ident(fn_arg: &FnArg) -> Ident {
    match fn_arg {
        FnArg::Receiver(_) => parse_quote!(object),
        FnArg::Typed(pat_type) => match pat_type.pat.as_ref() {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            _ => todo!(),
        },
    }
}

fn fn_arg_to_state_field(fn_arg: &FnArg) -> Expr {
    match fn_arg {
        FnArg::Receiver(_) => panic!(),
        FnArg::Typed(pat_type) => {
            let field_ty = match pat_type.ty.as_ref() {
                Type::Reference(reference) => reference.elem.clone(),
                _ => todo!(),
            };
            match pat_type.pat.as_ref() {
                Pat::Ident(pat_ident) => {
                    let field_ident = &pat_ident.ident;
                    parse_quote!(#field_ident: #field_ty)
                }
                _ => todo!(),
            }
        }
    }
}

fn fn_arg_to_superstate_field(fn_arg: &FnArg) -> Expr {
    match fn_arg {
        FnArg::Receiver(_) => panic!(),
        FnArg::Typed(pat_type) => {
            let field_ty = match pat_type.ty.as_ref() {
                Type::Reference(reference) => {
                    let mut reference = reference.clone();
                    reference.lifetime =
                        Some(Lifetime::new(SUPERSTATE_LIFETIME, Span::call_site()));
                    Type::Reference(reference)
                }
                _ => todo!(),
            };
            match pat_type.pat.as_ref() {
                Pat::Ident(pat_ident) => {
                    let field_ident = &pat_ident.ident;
                    parse_quote!(#field_ident: #field_ty)
                }
                _ => todo!(),
            }
        }
    }
}

fn snake_case_to_pascal_case(snake: &Ident) -> Ident {
    let mut pascal = String::new();
    for part in snake.to_string().split("_") {
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
    format_ident!("{}", pascal)
}

#[test]
fn valid_input() {
    use quote::quote;
    use syn::parse_quote;

    let object_ty = parse_quote!(Blinky);

    let item_impl = parse_quote!(
        impl Blinky { }
    );

    let state_name = parse_quote!(State);
    let state_derives = vec![parse_quote!(Copy), parse_quote!(Clone)];
    let superstate_name = parse_quote!(Superstate);
    let superstate_derives = vec![parse_quote!(Copy), parse_quote!(Clone)];
    let input = parse_quote!(input);
    let input_idents = vec![parse_quote!(input)];

    let state_machine = analyze::StateMachine {
        object_ty,
        state_name,
        state_derives,
        superstate_name,
        superstate_derives,
        input,
        input_idents,
    };

    let on_state = analyze::State {
        handler_name: parse_quote!(on),
        superstate: parse_quote!(Playing),
        entry_action: None,
        exit_action: None,
        inputs: vec![
            parse_quote!(&mut self),
            parse_quote!(event: &Event),
            parse_quote!(led: &mut bool),
            parse_quote!(counter: &mut usize),
        ],
        object_input: Some(parse_quote!(&mut self)),
        external_inputs: vec![parse_quote!(event: &Event)],
        state_inputs: vec![
            parse_quote!(led: &mut bool),
            parse_quote!(counter: &mut usize),
        ],
    };

    let off_state = analyze::State {
        handler_name: parse_quote!(off),
        superstate: parse_quote!(Playing),
        entry_action: None,
        exit_action: None,
        inputs: vec![
            parse_quote!(&mut self),
            parse_quote!(event: &Event),
            parse_quote!(led: &mut led),
            parse_quote!(counter: &mut counter),
        ],
        object_input: Some(parse_quote!(&mut self)),
        external_inputs: vec![parse_quote!(event: &Event)],
        state_inputs: vec![
            parse_quote!(led: &mut bool),
            parse_quote!(counter: &mut usize),
        ],
    };

    let playing_superstate = analyze::Superstate {
        handler_name: parse_quote!(playing),
        superstate: None,
        entry_action: None,
        exit_action: None,
        inputs: vec![
            parse_quote!(&mut self),
            parse_quote!(event: &Event),
            parse_quote!(led: &mut bool),
        ],
        object_input: Some(parse_quote!(&mut self)),
        external_inputs: vec![parse_quote!(event: &Event)],
        state_inputs: vec![parse_quote!(led: &mut bool)],
    };

    let enter_on_action = analyze::Action {
        handler_name: parse_quote!(enter_on),
        inputs: vec![parse_quote!(&mut self), parse_quote!(event: &Event)],
    };

    let mut states = HashMap::new();
    states.insert(on_state.handler_name.clone(), on_state);
    states.insert(off_state.handler_name.clone(), off_state);

    let mut superstates = HashMap::new();
    superstates.insert(playing_superstate.handler_name.clone(), playing_superstate);

    let mut actions = HashMap::new();
    actions.insert(enter_on_action.handler_name.clone(), enter_on_action);

    let model = Model {
        state_machine,
        item_impl,
        states,
        superstates,
        actions,
    };

    let actual = lower(model);

    // let pat = lower.pat;
    // let call = lower.handler_call;
    // let variant = lower.variant;

    // dbg!(quote!(#pat).to_string());
    // dbg!(quote!(#call).to_string());
    // dbg!(quote!(#variant).to_string());

    // let lower = lower_superstate(&superstate, &parse_quote!(Blinky));

    // let pat = lower.pat;
    // let call = lower.handler_call;
    // let variant = lower.variant;

    // dbg!(quote!(#pat).to_string());
    // dbg!(quote!(#call).to_string());
    // dbg!(quote!(#variant).to_string());

    // let action = analyze::Action {
    //     handler_name: parse_quote!(enter_on),
    //     inputs: vec![
    //         parse_quote!(&mut self),
    //         parse_quote!(counter: &mut usize),
    //         parse_quote!(led: &mut bool),
    //     ],
    // };

    // let lower = lower_action(&action, &parse_quote!(Blinky));

    // let call = lower.handler_call;

    // // let ir = Ir {
    // //     item_im
    // // };

    // dbg!(quote!(#call).to_string());
}

#[test]
fn example_unwrap() {
    fn test((test, hello): (usize, isize)) -> usize {}
}
