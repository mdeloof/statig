use proc_macro2::Span;
use proc_macro_error::abort;
use quote::format_ident;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use syn::parse::Parser;
use syn::{parse_quote, Expr, Field, ItemImpl, Lifetime, Type};
use syn::{ExprCall, FnArg, Ident, Pat, Path, Variant};
use syn::{ItemFn, Visibility};

use crate::analyze;
use crate::analyze::Model;

const SUPERSTATE_LIFETIME: &str = "'a";

/// Intermediate representation of the state machine.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct Ir {
    /// A copy of the item impl that was parsed.
    pub item_impl: ItemImpl,
    /// General information regarding the staet machine.
    pub state_machine: StateMachine,
    /// The states of the state machine.
    pub states: HashMap<Ident, State>,
    /// The superstate of the state machine.
    pub superstates: HashMap<Ident, Superstate>,
}

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
/// General information regarding the state machine.
pub struct StateMachine {
    /// Initial state.
    pub init_state: ExprCall,
    /// The type on which the state machine is implemented.
    pub context_ty: Type,
    /// The type of the event.
    pub event_ty: Type,
    /// The type of the state enum.
    pub state_ty: Type,
    /// Derives that will be applied on the state type.
    pub state_derives: Vec<Path>,
    /// The name of the superstate type (ex. `Superstate`)
    pub superstate_ident: Ident,
    /// The type of the superstate enum (ex. `Superstate<'a>`)
    pub superstate_ty: Type,
    /// Derives that will be applied to the superstate type.
    pub superstate_derives: Vec<Path>,
    pub on_transition: Option<Path>,
    pub on_dispatch: Option<Path>,
    /// The visibility for the derived types,
    pub visibility: Visibility,
    pub external_input_pattern: Pat,
}

/// Information regarding a state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    /// The variant that will be part of the state enum
    /// (e.g. `On { led: bool }`)
    pub variant: Variant,
    /// The pattern that we'll use to match on the state enum.
    /// (e.g. `State::On { led }`)
    pub pat: Pat,
    /// The call to the state handler
    /// (e.g. `Blinky::on(object, led, input)`).
    pub handler_call: ExprCall,
    /// The call to the entry action of the state, if defined
    /// (e.g. `Blinky::enter_on(object, led)`, `{}`, ..).
    pub entry_action_call: Expr,
    /// The call to the exit action of the state, if defined
    /// (e.g. `Blinky::exit_on(object, led)`, `{}`, ..).
    pub exit_action_call: Expr,
    /// The pattern to create the superstate variant.
    /// (e.g. `Some(Superstate::Playing { led })`, `None`, ..).
    pub superstate_pat: Pat,
    /// The constructor to create the state
    /// (e.g. `const fn on(led: bool) -> Self { Self::On { led }}`).
    pub constructor: ItemFn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Superstate {
    /// The variant that will be part of the superstate enum
    /// (e.g. `Playing { led: &'mut bool }`).
    pub variant: Variant,
    /// The pattern that we'll use to mactch on the superstate enum
    /// (e.g. `Superstate::Playing { led }`).
    pub pat: Pat,
    /// The call to the superstate handler
    /// (e.g. `Blinky::playing(object, led)`)
    pub handler_call: ExprCall,
    /// The call to the entry action of the superstate, if defined
    /// (e.g. `Blinky::enter_playing(object, led)`)
    pub entry_action_call: Expr,
    /// The call to the exit action of the superstate, if defined
    /// (e.g. `Blinky::exit_playing(object, led)`).
    pub exit_action_call: Expr,
    /// The pattern to create the superstate variant.
    /// (e.g. `Some(Superstate::Playing { led })`, `None`, ..).
    pub superstate_pat: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Action {
    /// The call to the action.
    /// (e.g. `Blinky::exit_off(object, led)`)
    pub handler_call: ExprCall,
}

pub fn lower(model: &Model) -> Ir {
    let item_impl = model.item_impl.clone();

    let init_state = model.state_machine.init_state.clone();

    let state_name = &model.state_machine.state_name;
    let state_ty = parse_quote!(#state_name);

    let superstate_ident = &model.state_machine.superstate_name;
    let mut superstate_ty = parse_quote!(#superstate_ident);

    let on_transition = model.state_machine.on_transition.clone();
    let on_dispatch = model.state_machine.on_dispatch.clone();

    let mut states: HashMap<Ident, State> = model
        .states
        .iter()
        .map(|(key, value)| (key.clone(), lower_state(value, &model.state_machine)))
        .collect();

    let mut superstates: HashMap<Ident, Superstate> = model
        .superstates
        .iter()
        .inspect(|(_, value)| {
            if !value.state_inputs.is_empty() {
                superstate_ty = parse_quote!(#superstate_ident <'a>);
            }
        })
        .map(|(key, value)| (key.clone(), lower_superstate(value, &model.state_machine)))
        .collect();

    let actions: HashMap<Ident, Action> = model
        .actions
        .iter()
        .map(|(key, value)| (key.clone(), lower_action(value, &model.state_machine)))
        .collect();

    // Linking states
    for (key, state) in &mut states {
        if let Some(superstate) = model
            .states
            .get(key)
            .and_then(|state| state.superstate.as_ref())
        {
            match superstates.get(superstate) {
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
            match actions.get(entry_action) {
                Some(action) => state.entry_action_call = action.handler_call.clone().into(),
                None => abort!(entry_action, "entry action not found"),
            }
        }

        if let Some(exit_action) = model
            .states
            .get(key)
            .and_then(|state| state.exit_action.as_ref())
        {
            match actions.get(exit_action) {
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
            match superstates_clone.get(superstate_superstate) {
                Some(superstate_superstate) => {
                    let superstate_superstate_pat = &superstate_superstate.pat;
                    if let Some(superstate) = superstates.get_mut(key) {
                        superstate.superstate_pat = parse_quote!(Some(#superstate_superstate_pat))
                    }
                }
                None => abort!(superstate_superstate, "superstate not found"),
            }
        }

        if let Some(entry_action) = model
            .superstates
            .get(key)
            .and_then(|state| state.entry_action.as_ref())
        {
            match actions.get(entry_action) {
                Some(action) => {
                    if let Some(superstate) = superstates.get_mut(key) {
                        superstate.entry_action_call = action.handler_call.clone().into();
                    }
                }
                None => abort!(entry_action, "action not found"),
            }
        }

        if let Some(exit_action) = model
            .superstates
            .get(key)
            .and_then(|state| state.exit_action.as_ref())
        {
            match actions.get(exit_action) {
                Some(action) => {
                    if let Some(superstate) = superstates.get_mut(key) {
                        superstate.exit_action_call = action.handler_call.clone().into();
                    }
                }
                None => abort!(exit_action, "action not found"),
            }
        }
    }

    // Finding event types
    let mut event_idents = model
        .state_machine
        .external_inputs
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let mut event_idents_types: HashMap<Ident, Type> = HashMap::new();

    for state in model.states.values() {
        for external_input in &state.external_inputs {
            if let FnArg::Typed(pat_type) = external_input {
                if let Pat::Ident(external_input_ident) = &*pat_type.pat {
                    if event_idents.remove(&external_input_ident.ident) {
                        let ty = match &*pat_type.ty {
                            Type::Reference(reference) => reference.elem.deref().clone(),
                            _ => todo!(),
                        };
                        event_idents_types.insert(external_input_ident.ident.clone(), ty);
                    }
                }
            }
        }
    }

    for superstates in model.superstates.values() {
        for external_input in &superstates.external_inputs {
            if let FnArg::Typed(pat_type) = external_input {
                if let Pat::Ident(external_input_ident) = &*pat_type.pat {
                    if event_idents.remove(&external_input_ident.ident) {
                        let ty = match &*pat_type.ty {
                            Type::Reference(reference) => reference.elem.deref().clone(),
                            _ => todo!(),
                        };
                        event_idents_types.insert(external_input_ident.ident.clone(), ty);
                    }
                }
            }
        }
    }

    let event_ty = match event_idents_types.is_empty() {
        true => parse_quote!(()),
        false => pat_to_type(
            &model.state_machine.external_input_pattern,
            &event_idents_types,
        ),
    };

    let context_ty = model.state_machine.context_ty.clone();
    let state_derives = model.state_machine.state_derives.clone();

    let superstate_ident = model.state_machine.superstate_name.clone();
    let superstate_derives = model.state_machine.superstate_derives.clone();

    let visibility = model.state_machine.visibility.clone();

    let external_input_pattern = model.state_machine.external_input_pattern.clone();

    let state_machine = StateMachine {
        init_state,
        context_ty,
        event_ty,
        state_ty,
        state_derives,
        superstate_ident,
        superstate_ty,
        superstate_derives,
        on_transition,
        on_dispatch,
        visibility,
        external_input_pattern,
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
    let context_ty = &state_machine.context_ty;
    let state_name = &state_machine.state_name;

    let mut variant_fields: Vec<_> = state
        .state_inputs
        .iter()
        .map(fn_arg_to_state_field)
        .collect();

    for field in &state.local_storage {
        match variant_fields.iter_mut().find(|f| f.ident == field.ident) {
            Some(item) => {
                *item = field.clone();
            }
            None => variant_fields.push(field.clone()),
        }
    }

    let pat_fields: Vec<Ident> = variant_fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().clone())
        .collect();
    let handler_inputs: Vec<Ident> = state.inputs.iter().map(fn_arg_to_ident).collect();

    let variant = parse_quote!(#variant_name { #(#variant_fields),* });
    let pat = parse_quote!(#state_name::#variant_name { #(#pat_fields),*});
    let constructor = parse_quote!(const fn #state_handler_name ( #(#variant_fields),* ) -> Self { Self::#variant_name { #(#pat_fields),*} });
    let handler_call = parse_quote!(#context_ty::#state_handler_name(#(#handler_inputs),*));
    let entry_action_call = parse_quote!({});
    let exit_action_call = parse_quote!({});
    let superstate_pat = parse_quote!(None);

    State {
        variant,
        pat,
        constructor,
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
    let context_ty = &state_machine.context_ty;
    let superstate_ty = &state_machine.superstate_name;

    let mut variant_fields: Vec<_> = superstate
        .state_inputs
        .iter()
        .map(fn_arg_to_superstate_field)
        .collect();

    for field in &superstate.local_storage {
        match variant_fields.iter_mut().find(|f| f.ident == field.ident) {
            Some(item) => {
                *item = field.clone();
            }
            None => variant_fields.push(field.clone()),
        }
    }

    let pat_fields: Vec<Ident> = variant_fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().clone())
        .collect();
    let handler_inputs: Vec<Ident> = superstate.inputs.iter().map(fn_arg_to_ident).collect();

    let variant = parse_quote!(#superstate_name { #(#variant_fields),* });
    let pat = parse_quote!(#superstate_ty::#superstate_name { #(#pat_fields),*});
    let handler_call = parse_quote!(#context_ty::#superstate_handler_name(#(#handler_inputs),*));
    let entry_action_call = parse_quote!({});
    let exit_action_call = parse_quote!({});
    let superstate_pat = parse_quote!(None);

    Superstate {
        variant,
        pat,
        handler_call,
        entry_action_call,
        exit_action_call,
        superstate_pat,
    }
}

pub fn lower_action(action: &analyze::Action, state_machine: &analyze::StateMachine) -> Action {
    let action_handler_name = &action.handler_name;
    let context_ty = &state_machine.context_ty;

    let mut call_inputs: Vec<Ident> = Vec::new();

    for input in &action.inputs {
        match input {
            FnArg::Receiver(_) => {
                call_inputs.insert(0, parse_quote!(context));
            }

            // Typed argument.
            FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                Pat::Ident(pat_ident) => {
                    let field_ident = &pat_ident.ident;
                    call_inputs.push(parse_quote!(#field_ident));
                }
                _ => panic!("all patterns should be verified to be idents"),
            },
        }
    }

    let handler_call = parse_quote!(#context_ty::#action_handler_name(#(#call_inputs),*));

    Action { handler_call }
}

fn fn_arg_to_ident(fn_arg: &FnArg) -> Ident {
    match fn_arg {
        FnArg::Receiver(_) => parse_quote!(context),
        FnArg::Typed(pat_type) => match pat_type.pat.as_ref() {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            _ => panic!("all patterns should be verified to be idents"),
        },
    }
}

fn fn_arg_to_state_field(fn_arg: &FnArg) -> Field {
    match fn_arg {
        FnArg::Receiver(_) => panic!("`self` can never be a state field"),
        FnArg::Typed(pat_type) => {
            let field_ty = match pat_type.ty.as_ref() {
                Type::Reference(reference) => reference.elem.clone(),
                _ => abort!(fn_arg, "input must be passed as a reference"),
            };
            match pat_type.pat.as_ref() {
                Pat::Ident(pat_ident) => {
                    let field_ident = &pat_ident.ident;
                    Field::parse_named
                        .parse2(quote::quote!(#field_ident: #field_ty))
                        .unwrap()
                }
                _ => panic!("all patterns should be verified to be idents"),
            }
        }
    }
}

fn fn_arg_to_superstate_field(fn_arg: &FnArg) -> Field {
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
                _ => abort!(fn_arg, "input must be passed as a reference"),
            };
            match pat_type.pat.as_ref() {
                Pat::Ident(pat_ident) => {
                    let field_ident = &pat_ident.ident;
                    Field::parse_named
                        .parse2(quote::quote!(#field_ident: #field_ty))
                        .unwrap()
                }
                _ => panic!("all patterns should be verified to be idents"),
            }
        }
    }
}

fn snake_case_to_pascal_case(snake: &Ident) -> Ident {
    let mut pascal = String::new();
    for part in snake.to_string().split('_') {
        let mut characters = part.chars();
        pascal.push_str(&characters.next().map_or_else(String::new, |c| {
            c.to_uppercase().chain(characters).collect()
        }));
    }
    format_ident!("{}", pascal)
}

fn pat_to_type(pat: &Pat, idents: &HashMap<Ident, Type>) -> Type {
    match pat {
        Pat::Box(pat) => {
            let ty = pat_to_type(&pat.pat, idents);
            parse_quote!(Box<#ty>)
        }
        Pat::Ident(pat) => match idents.get(&pat.ident) {
            Some(ty) => ty.clone(),
            None => {
                abort!(pat,
                    "ident could not be matched to type";
                    help = "make sure the ident is used in at least one state or superstate"
                )
            }
        },
        Pat::Lit(pat) => abort!(
            pat,
            "`literal` patterns are not supported";
            help = "pattern in function must be irrefutable"
        ),
        Pat::Macro(pat) => abort!(pat, "macro pattern not supported"),
        Pat::Or(pat) => abort!(
            pat,
            "`or` patterns are not supported";
            help = "pattern in function must be irrefutable"
        ),
        Pat::Path(pat) => abort!(
            pat,
            "`path` patterns are not supported";
            help = "pattern in function must be irrefutable"
        ),
        Pat::Range(pat) => abort!(
            pat,
            "`range` patterns are not supported";
            help = "pattern in function must be irrefutable"
        ),
        Pat::Reference(pat) => abort!(pat, "`reference` patterns are not supported"),
        Pat::Rest(pat) => abort!(
            pat,
            "`rest` patterns are not supported";
            help = "pattern in function must be irrefutable"
        ),
        Pat::Slice(pat) => abort!(
            pat,
            "`slice` patterns are not supported";
            help = "pattern in function must be irrefutable"
        ),
        Pat::Struct(pat) => {
            let ty = &pat.path;
            parse_quote!(#ty)
        }
        Pat::Tuple(tuple) => {
            let types: Vec<_> = tuple
                .elems
                .iter()
                .map(|pat| pat_to_type(pat, idents))
                .collect();
            parse_quote!((#(#types),*))
        }
        Pat::TupleStruct(pat) => {
            let ty = &pat.path;
            parse_quote!(#ty)
        }
        Pat::Type(pat) => pat.ty.deref().clone(),
        Pat::Verbatim(_) => abort!(pat, "`verbatim` patterns are not supported"),
        Pat::Wild(_) => abort!(pat, "`wildcard` patterns are not supported"),
        _ => todo!(),
    }
}

#[cfg(test)]
fn create_analyze_state_machine() -> analyze::StateMachine {
    analyze::StateMachine {
        init_state: parse_quote!(State::on()),
        context_ty: parse_quote!(Blinky),
        state_name: parse_quote!(State),
        state_derives: vec![parse_quote!(Copy), parse_quote!(Clone)],
        superstate_name: parse_quote!(Superstate),
        superstate_derives: vec![parse_quote!(Copy), parse_quote!(Clone)],
        on_transition: None,
        on_dispatch: None,
        external_input_pattern: parse_quote!(input),
        external_inputs: vec![parse_quote!(input)],
        visibility: parse_quote!(pub),
    }
}

#[cfg(test)]
fn create_lower_state_machine() -> StateMachine {
    StateMachine {
        init_state: parse_quote!(State::on()),
        context_ty: parse_quote!(Blinky),
        event_ty: parse_quote!(()),
        state_ty: parse_quote!(State),
        state_derives: vec![parse_quote!(Copy), parse_quote!(Clone)],
        superstate_ident: parse_quote!(Superstate),
        superstate_ty: parse_quote!(Superstate<'a>),
        superstate_derives: vec![parse_quote!(Copy), parse_quote!(Clone)],
        on_transition: None,
        on_dispatch: None,
        visibility: parse_quote!(pub),
        external_input_pattern: parse_quote!(input),
    }
}

#[cfg(test)]
fn create_analyze_state() -> analyze::State {
    analyze::State {
        handler_name: parse_quote!(on),
        superstate: parse_quote!(playing),
        entry_action: parse_quote!(enter_on),
        exit_action: None,
        local_storage: vec![],
        inputs: vec![
            parse_quote!(&mut self),
            parse_quote!(input: &Event),
            parse_quote!(led: &mut bool),
            parse_quote!(counter: &mut usize),
        ],
        context_input: Some(parse_quote!(&mut self)),
        external_inputs: vec![parse_quote!(event: &Event)],
        state_inputs: vec![
            parse_quote!(led: &mut bool),
            parse_quote!(counter: &mut usize),
        ],
    }
}

#[cfg(test)]
fn create_lower_state() -> State {
    State {
        variant: parse_quote!(On {
            led: bool,
            counter: usize
        }),
        pat: parse_quote!(State::On { led, counter }),
        handler_call: parse_quote!(Blinky::on(context, input, led, counter)),
        entry_action_call: parse_quote!({}),
        exit_action_call: parse_quote!({}),
        superstate_pat: parse_quote!(None),
        constructor: parse_quote!(
            const fn on(led: bool, counter: usize) -> Self {
                Self::On { led, counter }
            }
        ),
    }
}

#[cfg(test)]
fn create_linked_lower_state() -> State {
    let mut state = create_lower_state();
    state.superstate_pat = parse_quote!(Some(Superstate::Playing { led, counter }));
    state.entry_action_call = parse_quote!(Blinky::enter_on(context, led));
    state
}

#[cfg(test)]
fn create_analyze_superstate() -> analyze::Superstate {
    analyze::Superstate {
        handler_name: parse_quote!(playing),
        superstate: None,
        entry_action: None,
        exit_action: None,
        local_storage: vec![],
        inputs: vec![
            parse_quote!(&mut self),
            parse_quote!(input: &Event),
            parse_quote!(led: &mut bool),
            parse_quote!(counter: &mut usize),
        ],
        context_input: Some(parse_quote!(&mut self)),
        external_inputs: vec![parse_quote!(event: &Event)],
        state_inputs: vec![
            parse_quote!(led: &mut bool),
            parse_quote!(counter: &mut usize),
        ],
    }
}

#[cfg(test)]
fn create_lower_superstate() -> Superstate {
    Superstate {
        variant: parse_quote!(Playing {
            led: &'a mut bool,
            counter: &'a mut usize
        }),
        pat: parse_quote!(Superstate::Playing { led, counter }),
        handler_call: parse_quote!(Blinky::playing(context, input, led, counter)),
        entry_action_call: parse_quote!({}),
        exit_action_call: parse_quote!({}),
        superstate_pat: parse_quote!(None),
    }
}

#[cfg(test)]
fn create_analyze_action() -> analyze::Action {
    analyze::Action {
        handler_name: parse_quote!(enter_on),
        inputs: vec![parse_quote!(&mut self), parse_quote!(led: &mut bool)],
    }
}

#[cfg(test)]
fn create_lower_action() -> Action {
    Action {
        handler_call: parse_quote!(Blinky::enter_on(context, led)),
    }
}

#[cfg(test)]
fn create_analyze_model() -> analyze::Model {
    analyze::Model {
        item_impl: parse_quote!(impl Blinky { }),
        state_machine: create_analyze_state_machine(),
        states: [create_analyze_state()]
            .into_iter()
            .map(|state| (state.handler_name.clone(), state))
            .collect(),
        superstates: [create_analyze_superstate()]
            .into_iter()
            .map(|state| (state.handler_name.clone(), state))
            .collect(),
        actions: [create_analyze_action()]
            .into_iter()
            .map(|state| (state.handler_name.clone(), state))
            .collect(),
    }
}

#[cfg(test)]
fn create_lower_model() -> Ir {
    Ir {
        item_impl: parse_quote!(impl Blinky { }),
        state_machine: create_lower_state_machine(),
        states: [create_linked_lower_state()]
            .into_iter()
            .map(|state| (format_ident!("on"), state))
            .collect(),
        superstates: [create_lower_superstate()]
            .into_iter()
            .map(|state| (format_ident!("playing"), state))
            .collect(),
    }
}

#[test]
fn test_lower_state() {
    let analyze_state_machine = create_analyze_state_machine();
    let analyze_state = create_analyze_state();

    let actual = lower_state(&analyze_state, &analyze_state_machine);
    let expected = create_lower_state();

    assert_eq!(actual, expected);
}

#[test]
fn test_lower_superstate() {
    let analyze_state_machine = create_analyze_state_machine();
    let analyze_superstate = create_analyze_superstate();

    let actual = lower_superstate(&analyze_superstate, &analyze_state_machine);
    let expected = create_lower_superstate();

    assert_eq!(actual, expected);
}

#[test]
fn test_lower_action() {
    let state_machine = create_analyze_state_machine();
    let action = create_analyze_action();

    let actual = lower_action(&action, &state_machine);
    let expected = create_lower_action();

    assert_eq!(actual, expected);
}

#[test]
fn test_lower() {
    let model = create_analyze_model();

    let actual = lower(&model);
    let expected = create_lower_model();

    assert_eq!(actual, expected);
}

#[test]
fn test_pat_to_type() {
    let idents: HashMap<_, _> = [
        (parse_quote!(counter), parse_quote!(i32)),
        (parse_quote!(context), parse_quote!(Context)),
    ]
    .into();

    let pat = parse_quote!(Vec3 { x, y, z });

    let actual = pat_to_type(&pat, &idents);
    let expected = parse_quote!(Vec3);

    assert_eq!(actual, expected);

    let pat = parse_quote!((counter, context));

    let actual = pat_to_type(&pat, &idents);
    let expected = parse_quote!((i32, Context));

    assert_eq!(actual, expected);
}
