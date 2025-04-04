use std::collections::HashMap;

use proc_macro_error2::abort;
use syn::parse::Parser;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Expr, ExprLit, Field, FnArg, Generics, Ident, ImplItem, ImplItemFn, ItemImpl, Lit,
    LitStr, Meta, MetaList, Pat, PatIdent, PatType, Path, Token, Type, Visibility,
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
    /// The initial state of the state machine.
    pub initial_state: Expr,
    /// The type on which the state machine is implemented.
    pub shared_storage_type: Type,
    /// The path of the shared storage.
    pub shared_storage_path: Path,
    /// The generics associated with the shared storage type.
    pub shared_storage_generics: Generics,
    /// The name for the state type.
    pub state_ident: Ident,
    /// Derives that will be applied on the state type.
    pub state_derives: Vec<Path>,
    /// The name of the superstate type.
    pub superstate_ident: Ident,
    /// Derives that will be applied to the superstate type.
    pub superstate_derives: Vec<Path>,
    /// The identifier that is used for the event argument.
    pub event_ident: Ident,
    /// The identifier that is used for the context argument.
    pub context_ident: Ident,
    /// The visibility of the derived types.
    pub visibility: Visibility,
    /// Optional `before_transition` callback.
    pub before_transition: Option<Path>,
    /// Optional `after_transition` callback.
    pub after_transition: Option<Path>,
    /// Optional `before_dispatch` callback.
    pub before_dispatch: Option<Path>,
    /// Optional `after_dispatch` callback.
    pub after_dispatch: Option<Path>,
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
    /// Local storage,
    pub local_storage: Vec<Field>,
    /// Local storage default.
    pub local_storage_default: Vec<LocalStorageDefault>,
    /// Inputs required by the state handler.
    pub inputs: Vec<FnArg>,
    /// Inputs provided by the state-local storage.
    pub state_inputs: Vec<PatType>,
    /// Event that is submitted to the state machine.
    pub event_arg: Option<PatType>,
    /// Context that is submitted to the state machine.
    pub context_arg: Option<PatType>,
    /// Whether the function is async or not.
    pub is_async: bool,
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
    /// Local storage,
    pub local_storage: Vec<Field>,
    /// Inputs required by the superstate handler.
    pub inputs: Vec<FnArg>,
    /// Inputs provided by the state-local storage.
    pub state_inputs: Vec<PatType>,
    /// Event that is submitted to the state machine.
    pub event_arg: Option<PatType>,
    /// Context that is submitted to the state machine.
    pub context_arg: Option<PatType>,
    /// Whether the function is async or not.
    pub is_async: bool,
}

/// Information regarding an action.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct Action {
    /// Name of the action
    pub handler_name: Ident,
    /// Inputs required by the action handler.
    pub inputs: Vec<FnArg>,
    /// Whether the function is async or not.
    pub is_async: bool,
}

/// Information regarding a local storage default.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub enum LocalStorageDefault {
    Empty { ident: Ident },
    Value { ident: Ident, value: Expr },
}

impl LocalStorageDefault {
    pub(crate) fn ident(&self) -> &Ident {
        match self {
            LocalStorageDefault::Empty { ident } => ident,
            LocalStorageDefault::Value { ident, .. } => ident,
        }
    }
}

/// Analyze the impl block and create a model.
pub fn analyze(attribute_args: Vec<Meta>, mut item_impl: ItemImpl) -> Model {
    let state_machine = analyze_state_machine(&attribute_args[..], &item_impl);

    let mut states = HashMap::new();
    let mut superstates = HashMap::new();
    let mut actions = HashMap::new();

    // Create an iterator over only the method items.
    let methods = item_impl.items.iter_mut().filter_map(|item| match item {
        ImplItem::Fn(method) => Some(method),
        _ => None,
    });

    // Iterator over the methods in the impl block and check if they are marked
    // as a `#[state]`, `#[superstate]` or `#[action]`.
    for method in methods {
        let paths: Vec<_> = method
            .attrs
            .iter()
            .map(|attr| attr.meta.path().clone())
            .collect();
        for path in paths {
            if path.is_ident("state") {
                let state = analyze_state(method, &state_machine);
                states.insert(state.handler_name.clone(), state);
            } else if path.is_ident("superstate") {
                let superstate = analyze_superstate(method, &state_machine);
                superstates.insert(superstate.handler_name.clone(), superstate);
            } else if path.is_ident("action") {
                let action = analyze_action(method);
                actions.insert(action.handler_name.clone(), action);
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
pub fn analyze_state_machine(attribute_args: &[Meta], item_impl: &ItemImpl) -> StateMachine {
    let shared_storage_type = item_impl.self_ty.as_ref().clone();
    let shared_storage_generics = item_impl.generics.clone();
    let shared_storage_path = get_shared_storage_path(&shared_storage_type);

    let mut initial_state: Option<Expr> = None;

    let mut state_ident = parse_quote!(State);
    let mut state_derives = Vec::new();
    let mut superstate_ident = parse_quote!(Superstate);
    let mut superstate_derives = Vec::new();

    let mut after_transition = None;
    let mut before_transition = None;
    let mut before_dispatch = None;
    let mut after_dispatch = None;

    let mut visibility = parse_quote!(pub);
    let mut event_ident = parse_quote!(event);
    let mut context_ident = parse_quote!(context);

    let mut state_meta: MetaList = parse_quote!(state());
    let mut superstate_meta: MetaList = parse_quote!(superstate());

    // Iterate over the meta attributes on the `#[state_machine]` macro.
    for meta in attribute_args {
        if meta.path().is_ident("initial") {
            initial_state = match meta_require_name_lit_str(meta).parse() {
                Ok(initial_state) => Some(initial_state),
                Err(_) => abort!(meta, "initial state must be an expression"),
            }
        } else if meta.path().is_ident("event_identifier") {
            event_ident = match meta_require_name_lit_str(meta).parse() {
                Ok(event_ident) => event_ident,
                Err(_) => abort!(meta, "event identifier must be an ident"),
            }
        } else if meta.path().is_ident("context_identifier") {
            context_ident = match meta_require_name_lit_str(meta).parse() {
                Ok(context_ident) => context_ident,
                Err(_) => abort!(meta, "context identifier must be an ident"),
            }
        } else if meta.path().is_ident("before_transition") {
            before_transition = match meta_require_name_lit_str(meta).parse() {
                Ok(before_transition) => Some(before_transition),
                Err(_) => abort!(meta, "before transition hook must be a function pointer"),
            }
        } else if meta.path().is_ident("after_transition") {
            after_transition = match meta_require_name_lit_str(meta).parse() {
                Ok(after_transition) => Some(after_transition),
                Err(_) => abort!(meta, "after transition hook must be a function pointer"),
            }
        } else if meta.path().is_ident("before_dispatch") {
            before_dispatch = match meta_require_name_lit_str(meta).parse() {
                Ok(before_transition) => Some(before_transition),
                Err(_) => abort!(meta, "before dispatch hook must be a function pointer"),
            }
        } else if meta.path().is_ident("after_dispatch") {
            after_dispatch = match meta_require_name_lit_str(meta).parse() {
                Ok(after_dispatch) => Some(after_dispatch),
                Err(_) => abort!(meta, "after dispatch hook must be a function pointer"),
            }
        } else if meta.path().is_ident("visibility") {
            visibility = match meta_require_name_lit_str(meta).parse() {
                Ok(visibility) => visibility,
                Err(_) => abort!(meta, "visibility must be a visibility keyword"),
            }
        } else if meta.path().is_ident("state") {
            state_meta = match meta.require_list() {
                Ok(list) => list.clone(),
                Err(_) => abort!(meta, "state must contain a list of meta items"),
            }
        } else if meta.path().is_ident("superstate") {
            superstate_meta = match meta.require_list() {
                Ok(list) => list.clone(),
                Err(_) => abort!(meta, "superstate must contain a list of meta items"),
            }
        } else {
            abort!(meta, "unknown argument for `state_machine` attribute")
        }
    }

    // Check if there is an initial state given.
    let Some(initial_state) = initial_state else {
        abort!(
            initial_state,
            "no initial state defined";
            help = "add an initial state `#[state_machine(initial = \"State::initial_state()\"]"
        );
    };

    // Iterate over the meta attributes for the state enum.
    let state_meta =
        match Punctuated::<Meta, Token![,]>::parse_terminated.parse2(state_meta.tokens.clone()) {
            Ok(state_meta) => state_meta,
            Err(_) => abort!(state_meta, "state meta must be a list of meta items"),
        };

    for meta in state_meta.iter() {
        if meta.path().is_ident("name") {
            state_ident = match meta_require_name_lit_str(meta).parse() {
                Ok(state_ident) => state_ident,
                Err(_) => abort!(meta, "state type name must be an ident"),
            }
        } else if meta.path().is_ident("derive") {
            match meta.require_list().and_then(|list| {
                Punctuated::<Path, Token![,]>::parse_terminated.parse2(list.tokens.clone())
            }) {
                Ok(derives) => state_derives.extend(derives),
                Err(_) => abort!(meta, "derive must be a list of paths"),
            }
        } else {
            abort!(state_meta, "unknown argument for `state` attribute");
        }
    }

    // Iterate over the meta attributes for the superstate enum.
    let superstate_meta = match Punctuated::<Meta, Token![,]>::parse_terminated
        .parse2(superstate_meta.tokens.clone())
    {
        Ok(superstate_meta) => superstate_meta,
        Err(_) => abort!(
            superstate_meta,
            "superstate meta must be a list of meta items"
        ),
    };

    for meta in superstate_meta.iter() {
        if meta.path().is_ident("name") {
            superstate_ident = match meta_require_name_lit_str(meta).parse() {
                Ok(superstate_ident) => superstate_ident,
                Err(_) => abort!(meta, "superstate type name must be an ident"),
            }
        } else if meta.path().is_ident("derive") {
            match meta.require_list().and_then(|list| {
                Punctuated::<Path, Token![,]>::parse_terminated.parse2(list.tokens.clone())
            }) {
                Ok(derives) => superstate_derives.extend(derives),
                Err(_) => abort!(meta, "derive must be a list of paths"),
            }
        } else {
            abort!(
                superstate_meta,
                "unknown argument for `superstate` attribute"
            );
        }
    }

    StateMachine {
        initial_state,
        shared_storage_type,
        shared_storage_path,
        shared_storage_generics,
        state_ident,
        state_derives,
        superstate_ident,
        superstate_derives,
        before_dispatch,
        after_dispatch,
        before_transition,
        after_transition,
        event_ident,
        context_ident,
        visibility,
    }
}

/// Retrieve information regarding the state.
pub fn analyze_state(method: &mut ImplItemFn, state_machine: &StateMachine) -> State {
    let handler_name = method.sig.ident.clone();
    let inputs = method.sig.inputs.iter().cloned().collect();

    let mut superstate = None;
    let mut entry_action = None;
    let mut exit_action = None;
    let mut local_storage = Vec::new();
    let mut local_storage_default = Vec::new();
    let mut state_inputs = Vec::new();
    let mut event_arg = None;
    let mut context_arg = None;

    let generic_params = &method.sig.generics.params;
    if !generic_params.is_empty() {
        abort!(
            generic_params,
            "state handlers can not define their generics themselves";
            help = "consider declaring the generics on the impl block"
        )
    }

    let is_async = method.sig.asyncness.is_some();

    // Iterate over the inputs of the state handler.
    for input in &mut method.sig.inputs {
        match input {
            FnArg::Receiver(_) => (),
            FnArg::Typed(ref mut pat_type) => {
                let pat = pat_type.pat.as_mut();
                match pat {
                    Pat::Ident(pat) if state_machine.event_ident.eq(&pat.ident) => {
                        event_arg = Some(pat_type.clone());
                    }
                    Pat::Ident(pat) if state_machine.context_ident.eq(&pat.ident) => {
                        context_arg = Some(pat_type.clone());
                    }
                    Pat::Ident(ref mut input) => {
                        if let Some(index) = pat_type
                            .attrs
                            .iter()
                            .position(|attr| attr.path().is_ident("default"))
                        {
                            let attr = pat_type.attrs.swap_remove(index);
                            let default = analyze_local_storage_default(input, &attr);
                            local_storage_default.push(default);
                        }

                        state_inputs.push(pat_type.clone());
                    }
                    Pat::Reference(_) => {
                        state_inputs.push(pat_type.clone());
                    }
                    Pat::Tuple(_) => abort!(pat_type, "tuple pattern is not supported"),
                    Pat::TupleStruct(_) => {
                        abort!(pat_type, "tuple struct pattern is not supported")
                    }
                    Pat::Struct(_) => abort!(pat_type, "struct pattern is not supported"),
                    Pat::Wild(_) => abort!(
                        pat_type,
                        "wildcard pattern is not supported";
                        help = "consider giving the input a name"
                    ),
                    _ => abort!(pat_type, "patterns are not supported"),
                };
            }
        }
    }

    // Iterate over the meta attributes on the state handler.
    for meta in get_meta(&method.attrs, "state") {
        if meta.path().is_ident("superstate") {
            superstate = match meta_require_name_lit_str(&meta).parse() {
                Ok(superstate) => Some(superstate),
                Err(_) => abort!(meta, "superstate must be an ident"),
            }
        } else if meta.path().is_ident("entry_action") {
            entry_action = match meta_require_name_lit_str(&meta).parse() {
                Ok(entry_action) => Some(entry_action),
                Err(_) => abort!(meta, "entry action must be an ident"),
            }
        } else if meta.path().is_ident("exit_action") {
            exit_action = match meta_require_name_lit_str(&meta).parse() {
                Ok(exit_action) => Some(exit_action),
                Err(_) => abort!(meta, "exit action must be an ident"),
            }
        } else if meta.path().is_ident("local_storage") {
            match meta.require_list().and_then(|list| {
                Punctuated::<LitStr, Token![,]>::parse_terminated.parse2(list.tokens.clone())
            }) {
                Ok(fields) => {
                    for field in fields {
                        let field = match field.parse_with(Field::parse_named) {
                            Ok(field) => field,
                            Err(_) => abort!(field, "local storage entry must be a named field"),
                        };
                        local_storage.push(field);
                    }
                }
                Err(_) => abort!(meta, "local storage must be a list of string literals"),
            }
        }
    }

    State {
        handler_name,
        superstate,
        entry_action,
        exit_action,
        local_storage,
        local_storage_default,
        inputs,
        state_inputs,
        event_arg,
        context_arg,
        is_async,
    }
}

/// Retrieve the information regarding the superstate.
pub fn analyze_superstate(method: &ImplItemFn, state_machine: &StateMachine) -> Superstate {
    let handler_name = method.sig.ident.clone();
    let inputs = method.sig.inputs.iter().cloned().collect();

    let mut superstate = None;
    let mut entry_action = None;
    let mut exit_action = None;
    let mut local_storage = Vec::new();
    let mut state_inputs = Vec::new();
    let mut event_arg = None;
    let mut context_arg = None;

    let generic_params = &method.sig.generics.params;
    if !generic_params.is_empty() {
        abort!(
            generic_params,
            "superstate handlers can not define their generics themselves";
            help = "consider declaring the generics on the impl block"
        )
    }

    let is_async = method.sig.asyncness.is_some();

    // Iterate over the inputs of the superstate handler.
    for input in &method.sig.inputs {
        match input {
            FnArg::Receiver(_) => (),
            FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                Pat::Ident(pat) if state_machine.event_ident.eq(&pat.ident) => {
                    event_arg = Some(pat_type.clone());
                }
                Pat::Ident(pat) if state_machine.context_ident.eq(&pat.ident) => {
                    context_arg = Some(pat_type.clone());
                }
                Pat::Ident(_) => {
                    state_inputs.push(pat_type.clone());
                }
                Pat::Reference(_) => {
                    state_inputs.push(pat_type.clone());
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

    // Iterate over the meta attributes on the superstate handler.
    for meta in get_meta(&method.attrs, "superstate") {
        if meta.path().is_ident("superstate") {
            superstate = match meta_require_name_lit_str(&meta).parse() {
                Ok(superstate) => Some(superstate),
                Err(_) => abort!(meta, "superstate must be an ident"),
            }
        } else if meta.path().is_ident("entry_action") {
            entry_action = match meta_require_name_lit_str(&meta).parse() {
                Ok(entry_action) => Some(entry_action),
                Err(_) => abort!(meta, "entry action must be an ident"),
            }
        } else if meta.path().is_ident("exit_action") {
            exit_action = match meta_require_name_lit_str(&meta).parse() {
                Ok(exit_action) => Some(exit_action),
                Err(_) => abort!(meta, "exit action must be an ident"),
            }
        } else if meta.path().is_ident("local_storage") {
            match meta.require_list().and_then(|list| {
                Punctuated::<LitStr, Token![,]>::parse_terminated.parse2(list.tokens.clone())
            }) {
                Ok(fields) => {
                    for field in fields {
                        let field = match field.parse_with(Field::parse_named) {
                            Ok(field) => field,
                            Err(_) => abort!(field, "local storage entry must be a named field"),
                        };
                        local_storage.push(field);
                    }
                }
                Err(_) => abort!(meta, "local storage must be a list of string literals"),
            }
        }
    }

    Superstate {
        handler_name,
        superstate,
        entry_action,
        exit_action,
        local_storage,
        inputs,
        state_inputs,
        event_arg,
        context_arg,
        is_async,
    }
}

/// Retrieve the information regarding the action.
pub fn analyze_action(method: &ImplItemFn) -> Action {
    let handler_name = method.sig.ident.clone();
    let inputs = method.sig.inputs.clone().into_iter().collect();
    let is_async = method.sig.asyncness.is_some();

    let generic_params = &method.sig.generics.params;
    if !generic_params.is_empty() {
        abort!(
            generic_params,
            "action handlers can not define their generics themselves";
            help = "consider declaring the generics on the impl block"
        )
    }

    Action {
        handler_name,
        inputs,
        is_async,
    }
}

fn analyze_local_storage_default(input: &PatIdent, attribute: &Attribute) -> LocalStorageDefault {
    if let Ok(_) = attribute.meta.require_path_only() {
        LocalStorageDefault::Empty {
            ident: input.ident.clone(),
        }
    } else if let Ok(name_value) = attribute.meta.require_name_value() {
        if let Expr::Lit(ExprLit {
            lit: Lit::Str(string),
            ..
        }) = &name_value.value
        {
            let value = match string.parse() {
                Ok(value) => value,
                Err(_) => abort!(string, "value must be an expression"),
            };
            LocalStorageDefault::Value {
                ident: input.ident.clone(),
                value,
            }
        } else {
            abort!(name_value, "must be string literal")
        }
    } else {
        abort!(attribute, "wrong attribute format")
    }
}

/// Parse the attributes as a meta item.
pub fn get_meta(attrs: &[Attribute], name: &str) -> Vec<Meta> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident(name))
        .filter_map(|attr| {
            let meta = attr
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .ok()?;
            Some(meta)
        })
        .flatten()
        .collect()
}

/// Get the ident of the shared storage type.
pub fn get_shared_storage_path(ty: &Type) -> Path {
    match ty {
        Type::Path(path) => {
            let segments: Vec<_> = path.path.segments.iter().map(|seg| &seg.ident).collect();
            parse_quote!(#(#segments)::*)
        }
        _ => panic!("can not get path of shared storage"),
    }
}

/// Require that the meta item be a name-value pair, where the value
/// is a string literal (ex. `initial = "State::initial_state"`).
fn meta_require_name_lit_str(meta: &Meta) -> &LitStr {
    match meta {
        Meta::NameValue(name_value) => match &name_value.value {
            Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) => lit_str,
            _ => abort!(name_value.value, "expected string literal"),
        },
        _ => abort!(meta, "expected name-value pair"),
    }
}

#[test]
fn valid_state_analyze() {
    use syn::parse_quote;

    let attribute_args: Punctuated<Meta, Token![,]> = parse_quote!(
        initial = "State::on()",
        event_identifier = "event",
        state(derive(Copy, Clone)),
        superstate(derive(Copy, Clone))
    );

    let item_impl: ItemImpl = parse_quote!(
        impl Blinky {
            #[state(
                superstate = "playing",
                entry_action = "enter_on",
                exit_action = "enter_off"
            )]
            fn on(&mut self, event: &Event) -> Response<State> {
                Response::Handled
            }

            #[superstate]
            fn playing(&mut self, event: &Event) -> Response<State> {
                Response::Handled
            }

            #[action]
            fn enter_on(&mut self) {}

            #[action]
            fn enter_off(&mut self) {}
        }
    );

    let actual = analyze(attribute_args.into_iter().collect(), item_impl.clone());

    let initial_state = parse_quote!(State::on());

    let shared_storage_type = parse_quote!(Blinky);
    let shared_storage_path = parse_quote!(Blinky);
    let shared_storage_generics = parse_quote!();

    let state_ident = parse_quote!(State);
    let state_derives = vec![parse_quote!(Copy), parse_quote!(Clone)];
    let superstate_ident = parse_quote!(Superstate);
    let superstate_derives = vec![parse_quote!(Copy), parse_quote!(Clone)];
    let before_transition = None;
    let after_transition = None;
    let before_dispatch = None;
    let after_dispatch = None;
    let event_ident = parse_quote!(event);
    let context_ident = parse_quote!(context);
    let visibility = parse_quote!(pub);

    let state_machine = StateMachine {
        initial_state,
        shared_storage_type,
        shared_storage_path,
        shared_storage_generics,
        state_ident,
        state_derives,
        superstate_ident,
        superstate_derives,
        before_transition,
        after_transition,
        before_dispatch,
        after_dispatch,
        event_ident,
        context_ident,
        visibility,
    };

    let state = State {
        handler_name: parse_quote!(on),
        superstate: parse_quote!(playing),
        entry_action: parse_quote!(enter_on),
        exit_action: parse_quote!(enter_off),
        local_storage: vec![],
        local_storage_default: vec![],
        inputs: vec![parse_quote!(&mut self), parse_quote!(event: &Event)],
        state_inputs: vec![],
        event_arg: Some(if let FnArg::Typed(event) = parse_quote!(event: &Event) {
            event
        } else {
            return;
        }),
        context_arg: None,
        is_async: false,
    };

    let superstate = Superstate {
        handler_name: parse_quote!(playing),
        superstate: None,
        entry_action: None,
        exit_action: None,
        local_storage: vec![],
        inputs: vec![parse_quote!(&mut self), parse_quote!(event: &Event)],
        state_inputs: vec![],
        event_arg: Some(if let FnArg::Typed(event) = parse_quote!(event: &Event) {
            event
        } else {
            return;
        }),
        context_arg: None,
        is_async: false,
    };

    let entry_action = Action {
        handler_name: parse_quote!(enter_on),
        inputs: vec![parse_quote!(&mut self)],
        is_async: false,
    };

    let exit_action = Action {
        handler_name: parse_quote!(enter_off),
        inputs: vec![parse_quote!(&mut self)],
        is_async: false,
    };

    let mut states = HashMap::new();
    let mut superstates = HashMap::new();
    let mut actions = HashMap::new();

    states.insert(state.handler_name.clone(), state);
    superstates.insert(superstate.handler_name.clone(), superstate);
    actions.insert(entry_action.handler_name.clone(), entry_action);
    actions.insert(exit_action.handler_name.clone(), exit_action);

    let expected = Model {
        item_impl,
        state_machine,
        states,
        superstates,
        actions,
    };

    assert_eq!(actual, expected);
}
