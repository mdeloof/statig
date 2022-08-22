use std::collections::HashMap;

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::format_ident;
use syn::{parse_quote, Visibility};
use syn::{
    Attribute, AttributeArgs, FnArg, Ident, ImplItem, ImplItemMethod, ItemImpl, Lit, Meta,
    NestedMeta, Pat, Path, Type,
};

/// Model of the state machine.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct Model {
    /// A copy of the item impl that was parsed.
    pub item_impl: ItemImpl,
    /// General information regarding the state machine.
    pub state_machine: StateMachine,
    /// The states of the state machine.
    pub states: HashMap<Ident, State>,
    /// The superstates of the state machine.
    pub superstates: HashMap<Ident, Superstate>,
    /// The actions of the state machine.
    pub actions: HashMap<Ident, Action>,
}

/// General information regarding the state machine.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct StateMachine {
    /// The type on which the state machine is implemented.
    pub object_ty: Type,
    /// The name for the state type.
    pub state_name: Ident,
    /// Derives that will be applied on the state type.
    pub state_derives: Vec<Path>,
    /// The name of the superstate type.
    pub superstate_name: Ident,
    /// Derives that will be applied to the superstate type.
    pub superstate_derives: Vec<Path>,
    /// The input that will be handled by the state machine.
    pub external_input_pattern: Pat,
    /// The idents that will be binded by destructering the input pattern.
    pub external_inputs: Vec<Ident>,
    /// The visibility of the derived types.
    pub visibility: Visibility,
}

/// Information regarding a state.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct State {
    /// Name of the state.
    pub handler_name: Ident,
    /// Optional superstate.
    pub superstate: Option<Ident>,
    /// Optional entry action.
    pub entry_action: Option<Ident>,
    /// Optional exit action.
    pub exit_action: Option<Ident>,
    /// Inputs required by the state handler.
    pub inputs: Vec<FnArg>,
    /// Optional receiver input for the state handler (e.g. `&mut self`).
    pub object_input: Option<FnArg>,
    /// Inputs provided by the state-local storage.
    pub state_inputs: Vec<FnArg>,
    /// Inputs that are submitted to the state machine.
    pub external_inputs: Vec<FnArg>,
}

/// Information regarding a superstate.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct Superstate {
    /// Name of the superstate.
    pub handler_name: Ident,
    /// Optional superstate.
    pub superstate: Option<Ident>,
    /// Optional entry action.
    pub entry_action: Option<Ident>,
    /// Optional exit action.
    pub exit_action: Option<Ident>,
    /// Inputs required by the superstate handler.
    pub inputs: Vec<FnArg>,
    /// Optional receiver input for the state handler (e.g. `&mut self`).
    pub object_input: Option<FnArg>,
    /// Inputs provided by the state-local storage.
    pub state_inputs: Vec<FnArg>,
    /// Inputs that are submitted to the state machine.
    pub external_inputs: Vec<FnArg>,
}

/// Information regarding an action.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct Action {
    /// Name of the action
    pub handler_name: Ident,
    /// Inputs required by the action handler.
    pub inputs: Vec<FnArg>,
}

/// Analyze the impl block and create a model.
pub fn analyze(attribute_args: AttributeArgs, item_impl: ItemImpl) -> Model {
    let mut states = HashMap::new();
    let mut superstates = HashMap::new();
    let mut actions = HashMap::new();

    let state_machine = analyze_state_machine(&attribute_args, &item_impl);

    // Iterate over the methods that are part of the impl block.
    for method in item_impl.items.iter().filter_map(|item| match item {
        ImplItem::Method(method) => Some(method),
        _ => None,
    }) {
        for attr in method.attrs.iter() {
            match &attr.path {
                path if path.is_ident("state") => {
                    let state = analyze_state(method, &state_machine);
                    states.insert(state.handler_name.clone(), state);
                }

                path if path.is_ident("superstate") => {
                    let superstate = analyze_superstate(method, &state_machine);
                    superstates.insert(superstate.handler_name.clone(), superstate);
                }

                path if path.is_ident("action") => {
                    let action = analyze_action(method);
                    actions.insert(action.handler_name.clone(), action);
                }

                _ => (),
            }
        }
    }

    Model {
        item_impl,
        state_machine,
        states,
        superstates,
        actions,
    }
}

/// Retrieve the top level settings of the state machine.
pub fn analyze_state_machine(attribute_args: &AttributeArgs, item_impl: &ItemImpl) -> StateMachine {
    let object_ty = item_impl.self_ty.as_ref().clone();

    let mut state_name = parse_quote!(State);
    let mut state_derives = Vec::new();
    let mut superstate_name = parse_quote!(Superstate);
    let mut superstate_derives = Vec::new();

    let mut visibility = parse_quote!(pub);
    let mut external_input_pattern = parse_quote!(input);

    for arg in attribute_args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(name_value)) if name_value.path.is_ident("input") => {
                match &name_value.lit {
                    Lit::Str(input_pat) => {
                        let token_stream: TokenStream = str::parse(&input_pat.value()).unwrap();
                        external_input_pattern = parse_quote!(#token_stream);
                    }
                    _ => abort!(name_value, "must be a string literal"),
                }
            }
            NestedMeta::Meta(Meta::NameValue(name_value))
                if name_value.path.is_ident("visibility") =>
            {
                match &name_value.lit {
                    Lit::Str(input_pat) => {
                        let token_stream: TokenStream = str::parse(&input_pat.value()).unwrap();
                        visibility = parse_quote!(#token_stream);
                    }
                    _ => abort!(name_value, "must be a string literal"),
                }
            }
            _ => abort!(arg, "argument not recognized"),
        }
    }

    let external_inputs = get_idents_from_pat(&external_input_pattern);

    let meta = get_meta(&item_impl.attrs, "state");

    for meta in meta {
        match meta {
            // Get the custom name for the state enum.
            Meta::NameValue(name_value) if name_value.path.is_ident("name") => {
                match name_value.lit {
                    Lit::Str(str_lit) => {
                        state_name = format_ident!("{}", str_lit.value());
                    }
                    _ => abort!(name_value, "expected string literal"),
                }
            }

            // Get the derives for the state enum.
            Meta::List(meta_list) if meta_list.path.is_ident("derive") => {
                for nested_meta in &meta_list.nested {
                    match nested_meta {
                        NestedMeta::Meta(meta) => {
                            state_derives.push(meta.path().clone());
                        }
                        _ => abort!(nested_meta, "expected list of traits"),
                    }
                }
            }

            // Other attributes are not recognized.
            _ => abort!(meta, "unknown attribute"),
        }
    }

    let meta = get_meta(&item_impl.attrs, "superstate");

    for meta in meta {
        match meta {
            // Get the custom name for the superstate enum.
            Meta::NameValue(name_value) if name_value.path.is_ident("name") => {
                match name_value.lit {
                    Lit::Str(str_lit) => {
                        superstate_name = format_ident!("{}", str_lit.value());
                    }
                    _ => abort!(name_value, "expected string literal"),
                }
            }

            // Get the derives of the superstate enum.
            Meta::List(meta_list) if meta_list.path.is_ident("derive") => {
                for nested_meta in &meta_list.nested {
                    match nested_meta {
                        NestedMeta::Meta(meta) => {
                            superstate_derives.push(meta.path().clone());
                        }
                        _ => abort!(nested_meta, "expected list of traits"),
                    }
                }
            }

            // Other attributes are not recognized.
            _ => abort!(meta, "unknown attribute"),
        }
    }

    StateMachine {
        object_ty,
        state_name,
        state_derives,
        superstate_name,
        superstate_derives,
        external_input_pattern,
        external_inputs,
        visibility,
    }
}

/// Retrieve information regarding the state.
pub fn analyze_state(method: &ImplItemMethod, state_machine: &StateMachine) -> State {
    let handler_name = method.sig.ident.clone();
    let inputs = method.sig.inputs.iter().cloned().collect();

    let mut superstate = None;
    let mut entry_action = None;
    let mut exit_action = None;
    let mut object_input = None;
    let mut state_inputs = Vec::new();
    let mut external_inputs = Vec::new();

    for input in &method.sig.inputs {
        match input {
            FnArg::Receiver(_) => object_input = Some(input.clone()),
            FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                Pat::Ident(pat) if state_machine.external_inputs.contains(&pat.ident) => {
                    external_inputs.push(input.clone());
                }
                Pat::Ident(_) => {
                    state_inputs.push(input.clone());
                }
                Pat::Reference(_) => {
                    state_inputs.push(input.clone());
                }
                Pat::Tuple(_) => abort!(pat_type, "tuple pattern is not supported"),
                Pat::TupleStruct(_) => abort!(pat_type, "tuple struct pattern is not supported"),
                Pat::Struct(_) => abort!(pat_type, "struct pattern is not supported"),
                Pat::Wild(_) => abort!(
                    pat_type,
                    "wildcard pattern is not supported";
                    help = "consider giving the input a name"
                ),
                _ => abort!(pat_type, "patterns are not supported"),
            },
        }
    }

    let meta = get_meta(&method.attrs, "state");

    for meta in meta {
        match meta {
            Meta::NameValue(name_value) if name_value.path.is_ident("superstate") => {
                if let Lit::Str(value) = name_value.lit {
                    superstate = Some(Ident::new(&value.value(), value.span()));
                }
            }
            Meta::NameValue(name_value) if name_value.path.is_ident("entry_action") => {
                if let Lit::Str(value) = name_value.lit {
                    entry_action = Some(Ident::new(&value.value(), value.span()));
                }
            }
            Meta::NameValue(name_value) if name_value.path.is_ident("exit_action") => {
                if let Lit::Str(value) = name_value.lit {
                    exit_action = Some(Ident::new(&value.value(), value.span()));
                }
            }
            _ => abort!(meta, "unknown attribute"),
        }
    }

    State {
        handler_name,
        superstate,
        entry_action,
        exit_action,
        inputs,
        object_input,
        state_inputs,
        external_inputs,
    }
}

/// Retrieve the information regarding the superstate.
pub fn analyze_superstate(method: &ImplItemMethod, state_machine: &StateMachine) -> Superstate {
    let handler_name = method.sig.ident.clone();
    let inputs = method.sig.inputs.iter().cloned().collect();

    let mut superstate = None;
    let mut entry_action = None;
    let mut exit_action = None;
    let mut object_input = None;
    let mut state_inputs = Vec::new();
    let mut external_inputs = Vec::new();

    for input in &method.sig.inputs {
        match input {
            FnArg::Receiver(_) => object_input = Some(input.clone()),
            FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                Pat::Ident(pat) if state_machine.external_inputs.contains(&pat.ident) => {
                    external_inputs.push(input.clone());
                }
                Pat::Ident(_) => {
                    state_inputs.push(input.clone());
                }
                Pat::Reference(_) => {
                    state_inputs.push(input.clone());
                }
                Pat::Tuple(_) => abort!(pat_type, "tuple pattern is not supported"),
                Pat::TupleStruct(_) => abort!(pat_type, "tuple struct pattern is not supported"),
                Pat::Struct(_) => abort!(pat_type, "struct pattern is not supported"),
                Pat::Wild(_) => abort!(
                    pat_type,
                    "wildcard pattern is not supported";
                    help = "consider giving the input a name"
                ),
                _ => abort!(pat_type, "patterns are not supported"),
            },
        }
    }

    let meta = get_meta(&method.attrs, "superstate");

    for meta in meta {
        match meta {
            Meta::NameValue(name_value) if name_value.path.is_ident("superstate") => {
                if let Lit::Str(value) = name_value.lit {
                    superstate = Some(Ident::new(&value.value(), value.span()));
                }
            }
            Meta::NameValue(name_value) if name_value.path.is_ident("entry_action") => {
                if let Lit::Str(value) = name_value.lit {
                    entry_action = Some(Ident::new(&value.value(), value.span()));
                }
            }
            Meta::NameValue(name_value) if name_value.path.is_ident("exit_action") => {
                if let Lit::Str(value) = name_value.lit {
                    exit_action = Some(Ident::new(&value.value(), value.span()));
                }
            }
            _ => abort!(meta, "unknown attribute"),
        }
    }

    Superstate {
        handler_name,
        superstate,
        entry_action,
        exit_action,
        inputs,
        object_input,
        state_inputs,
        external_inputs,
    }
}

/// Retrieve the information regarding the action.
pub fn analyze_action(method: &ImplItemMethod) -> Action {
    let handler_name = method.sig.ident.clone();
    let inputs = method.sig.inputs.clone().into_iter().collect();

    Action {
        handler_name,
        inputs,
    }
}

/// Parse the attributes as a meta item.
pub fn get_meta(attrs: &[Attribute], name: &str) -> Vec<Meta> {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident(name))
        .filter_map(|attr| attr.parse_meta().ok())
        .filter_map(|meta| match meta {
            Meta::List(list_meta) => Some(list_meta.nested),
            _ => None,
        })
        .flatten()
        .filter_map(|nested| match nested {
            NestedMeta::Meta(meta) => Some(meta),
            _ => None,
        })
        .collect()
}

/// Destructure a pattern and get the idents that will be bound.
pub fn get_idents_from_pat(pat: &Pat) -> Vec<Ident> {
    match pat {
        Pat::Ident(pat_ident) => vec![pat_ident.ident.clone()],
        Pat::Tuple(pat_tuple) => pat_tuple
            .elems
            .iter()
            .flat_map(get_idents_from_pat)
            .collect(),
        Pat::TupleStruct(pat_struct) => pat_struct
            .pat
            .elems
            .iter()
            .flat_map(get_idents_from_pat)
            .collect(),
        Pat::Struct(pat_struct) => pat_struct
            .fields
            .iter()
            .flat_map(|field| get_idents_from_pat(field.pat.as_ref()))
            .collect(),
        Pat::Range(_) => vec![],
        _ => abort!(pat, "pattern type is not supported"),
    }
}

#[test]
fn valid_state_analyze() {
    use syn::parse_quote;

    let attribute_arg: NestedMeta = parse_quote!(input = "input");
    let attribute_args = vec![attribute_arg];

    let item_impl: ItemImpl = parse_quote!(
        #[state_machine]
        #[state(derive(Copy, Clone))]
        #[superstate(derive(Copy, Clone))]
        impl Blinky {
            #[state(
                superstate = "playing",
                entry_action = "enter_on",
                exit_action = "enter_off"
            )]
            fn on(&mut self, input: &Event) -> Response<State> {
                Response::Handled
            }

            #[superstate]
            fn playing(&mut self, input: &Event) -> Response<State> {
                Response::Handled
            }

            #[action]
            fn enter_on(&mut self) {}
        }
    );

    let actual = analyze(attribute_args, item_impl.clone());

    let object_ty = parse_quote!(Blinky);

    let state_name = parse_quote!(State);
    let state_derives = vec![parse_quote!(Copy), parse_quote!(Clone)];
    let superstate_name = parse_quote!(Superstate);
    let superstate_derives = vec![parse_quote!(Copy), parse_quote!(Clone)];
    let external_input_pattern = parse_quote!(input);
    let external_inputs = vec![parse_quote!(input)];
    let visibility = parse_quote!(pub);

    let state_machine = StateMachine {
        object_ty,
        state_name,
        state_derives,
        superstate_name,
        superstate_derives,
        external_input_pattern,
        external_inputs,
        visibility,
    };

    let state = State {
        handler_name: parse_quote!(on),
        superstate: parse_quote!(playing),
        entry_action: parse_quote!(enter_on),
        exit_action: parse_quote!(enter_off),
        inputs: vec![parse_quote!(&mut self), parse_quote!(input: &Event)],
        object_input: Some(parse_quote!(&mut self)),
        state_inputs: vec![],
        external_inputs: vec![parse_quote!(input: &Event)],
    };

    let superstate = Superstate {
        handler_name: parse_quote!(playing),
        superstate: None,
        entry_action: None,
        exit_action: None,
        inputs: vec![parse_quote!(&mut self), parse_quote!(input: &Event)],
        object_input: Some(parse_quote!(&mut self)),
        state_inputs: vec![],
        external_inputs: vec![parse_quote!(input: &Event)],
    };

    let action = Action {
        handler_name: parse_quote!(enter_on),
        inputs: vec![parse_quote!(&mut self)],
    };

    let mut states = HashMap::new();
    let mut superstates = HashMap::new();
    let mut actions = HashMap::new();

    states.insert(state.handler_name.clone(), state);
    superstates.insert(superstate.handler_name.clone(), superstate);
    actions.insert(action.handler_name.clone(), action);

    let expected = Model {
        item_impl,
        state_machine,
        states,
        superstates,
        actions,
    };

    assert_eq!(actual, expected);
}
