use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, Arm, ItemEnum, ItemFn, ItemImpl, Lifetime, Variant};

use crate::lower::{Ir, Mode};
use crate::{CONTEXT_LIFETIME, EVENT_LIFETIME, SUPERSTATE_LIFETIME};

pub fn codegen(ir: Ir) -> TokenStream {
    let item_impl = &ir.item_impl;

    let state_machine_impl = codegen_state_machine_impl(&ir);

    let state_enum = codegen_state(&ir);
    let state_impl = codegen_state_impl(&ir);
    let state_impl_state = codegen_state_impl_state(&ir);
    let superstate_enum = codegen_superstate(&ir);
    let superstate_impl = codegen_superstate_impl_superstate(&ir);

    quote!(
        // Import the proc_macro attributes so they can be used to tag functions.
        use statig::{state, superstate, action};

        #item_impl

        #state_machine_impl

        #state_enum

        #state_impl

        #state_impl_state

        #superstate_enum

        #superstate_impl
    )
}

fn codegen_state_machine_impl(ir: &Ir) -> ItemImpl {
    let shared_storage_type = &ir.state_machine.shared_storage_type;
    let (impl_generics, _, where_clause) =
        &ir.state_machine.shared_storage_generics.split_for_impl();
    let event_type = &ir.state_machine.event_type;
    let context_type = &ir.state_machine.context_type;
    let state_ident = &ir.state_machine.state_ident;
    let (_, state_generics, _) = &ir.state_machine.state_generics.split_for_impl();
    let superstate_ident = &ir.state_machine.superstate_ident;
    let (_, superstate_generics, _) = &ir.state_machine.superstate_generics.split_for_impl();
    let superstate_lifetime = Lifetime::new(SUPERSTATE_LIFETIME, Span::call_site());
    let event_lifetime = Lifetime::new(EVENT_LIFETIME, Span::call_site());
    let context_lifetime = Lifetime::new(CONTEXT_LIFETIME, Span::call_site());

    let initial_state = &ir.state_machine.initial_state;

    let mode = match ir.state_machine.mode {
        Mode::Blocking => quote!(blocking),
        Mode::Awaitable => quote!(awaitable),
    };

    let before_dispatch = match &ir.state_machine.before_dispatch {
        None => quote!(),
        Some(before_dispatch) => match ir.state_machine.mode {
            Mode::Blocking => quote!(
                const BEFORE_DISPATCH: fn(
                    &mut Self,
                    StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
                    &Self::Event<'_>,
                ) = #before_dispatch;
            ),
            Mode::Awaitable => quote!(
                fn before_dispatch(
                    &mut self,
                    state_or_superstate: StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
                    event: &Self::Event<'_>,
                ) -> impl core::future::Future<Output = ()> {
                    #before_dispatch(self, state_or_superstate, event)
                }
            ),
        },
    };

    let after_dispatch = match &ir.state_machine.after_dispatch {
        None => quote!(),
        Some(after_dispatch) => match ir.state_machine.mode {
            Mode::Blocking => quote!(
                const AFTER_DISPATCH: fn(
                    &mut Self,
                    StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
                    &Self::Event<'_>,
                ) = #after_dispatch;
            ),
            Mode::Awaitable => quote!(
                fn after_dispatch(
                    &mut self,
                    state_or_superstate: StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
                    event: &Self::Event<'_>,
                ) -> impl core::future::Future<Output = ()> {
                    #after_dispatch(self, state_or_superstate, event)
                }
            ),
        },
    };

    let before_transition = match &ir.state_machine.before_transition {
        None => quote!(),
        Some(before_transition) => match ir.state_machine.mode {
            Mode::Blocking => quote!(
                const BEFORE_TRANSITION: fn(&mut Self, &Self::State, &Self::State) = #before_transition;
            ),
            Mode::Awaitable => quote!(
                fn before_transition(
                    &mut self,
                    source: &Self::State,
                    target: &Self::State,
                ) -> impl core::future::Future<Output = ()> {
                    #before_transition(self, source, target)
                }
            ),
        },
    };

    let after_transition = match &ir.state_machine.after_transition {
        None => quote!(),
        Some(after_transition) => match ir.state_machine.mode {
            Mode::Blocking => quote!(
                const AFTER_TRANSITION: fn(&mut Self, &Self::State, &Self::State) = #after_transition;
            ),
            Mode::Awaitable => quote!(
                fn after_transition(
                    &mut self,
                    source: &Self::State,
                    target: &Self::State,
                ) -> impl core::future::Future<Output = ()> {
                    #after_transition(self, source, target)
                }
            ),
        },
    };

    parse_quote!(
        impl #impl_generics statig::#mode::IntoStateMachine for #shared_storage_type #where_clause
        {
            type Event<#event_lifetime> = #event_type;
            type Context<#context_lifetime> = #context_type;
            type State = #state_ident #state_generics;
            type Superstate<#superstate_lifetime> = #superstate_ident #superstate_generics ;

            fn initial() -> #state_ident #state_generics {
                #initial_state
            }

            #before_transition
            #after_transition

            #before_dispatch
            #after_dispatch
        }
    )
}

fn codegen_state(ir: &Ir) -> ItemEnum {
    let state_ident = &ir.state_machine.state_ident;
    let (state_generics, _, _) = &ir.state_machine.state_generics.split_for_impl();
    let state_derives = &ir.state_machine.state_derives;

    let variants: Vec<Variant> = ir
        .states
        .values()
        .map(|state| state.variant.clone())
        .collect();
    let visibility = &ir.state_machine.visibility;

    parse_quote!(
        #[derive(#(#state_derives),*)]
        # visibility enum #state_ident #state_generics {
            #(#variants),*
        }
    )
}

fn codegen_state_impl(ir: &Ir) -> ItemImpl {
    let state_ident = &ir.state_machine.state_ident;
    let (impl_generics, state_generics, _) = &ir.state_machine.state_generics.split_for_impl();

    let constructors: Vec<ItemFn> = ir
        .states
        .values()
        .map(|state| &state.constructor)
        .cloned()
        .collect();

    parse_quote!(
        impl #impl_generics #state_ident #state_generics {
            #(#constructors)*
        }
    )
}

fn codegen_state_impl_state(ir: &Ir) -> ItemImpl {
    let shared_storage_type = &ir.state_machine.shared_storage_type;
    let (impl_generics, _, where_clause) = &ir.state_machine.state_impl_generics.split_for_impl();
    let state_ident = &ir.state_machine.state_ident;
    let (_, state_generics, _) = &ir.state_machine.state_generics.split_for_impl();
    let event_ident = &ir.state_machine.event_ident;
    let context_ident = &ir.state_machine.context_ident;

    let mut constructors: Vec<ItemFn> = Vec::new();
    let mut call_handler_arms: Vec<Arm> = Vec::new();
    let mut call_entry_action_arms: Vec<Arm> = Vec::new();
    let mut call_exit_action_arms: Vec<Arm> = Vec::new();
    let mut superstate_arms: Vec<Arm> = Vec::new();
    let mut same_state_arms: Vec<Arm> = Vec::new();

    for state in ir.states.values() {
        let pat = &state.pat;
        let handler_call = &state.handler_call;
        let entry_action_call = &state.entry_action_call;
        let exit_action_call = &state.exit_action_call;
        let superstate_pat = &state.superstate_pat;

        constructors.push(state.constructor.clone());
        call_handler_arms.push(parse_quote!(#pat => #handler_call));
        call_entry_action_arms.push(parse_quote!(#pat => #entry_action_call));
        call_exit_action_arms.push(parse_quote!(#pat => #exit_action_call));
        superstate_arms.push(parse_quote!(#pat => #superstate_pat));
    }

    call_handler_arms.push(parse_quote!(_ => statig::Response::Super));
    call_entry_action_arms.push(parse_quote!(_ => {}));
    call_exit_action_arms.push(parse_quote!(_ => {}));
    superstate_arms.push(parse_quote!(_ => None));
    same_state_arms.push(parse_quote!(_ => false));

    match ir.state_machine.mode {
        Mode::Blocking => {
            parse_quote!(
                #[allow(unused)]
                impl #impl_generics statig::blocking::State<#shared_storage_type> for #state_ident #state_generics #where_clause
                {
                    fn call_handler(
                        &mut self,
                        shared_storage: &mut #shared_storage_type,
                        #event_ident: &<#shared_storage_type as statig::blocking::IntoStateMachine>::Event<'_>,
                        #context_ident: &mut <#shared_storage_type as statig::blocking::IntoStateMachine>::Context<'_>
                    ) -> statig::Response<Self> where Self: Sized {
                        match self {
                            #(#call_handler_arms),*
                        }
                    }

                    fn call_entry_action(
                        &mut self,
                        shared_storage: &mut #shared_storage_type,
                        #context_ident: &mut <#shared_storage_type as statig::blocking::IntoStateMachine>::Context<'_>
                    ) {
                        match self {
                            #(#call_entry_action_arms),*
                        }
                    }

                    fn call_exit_action(
                        &mut self,
                        shared_storage: &mut #shared_storage_type,
                        #context_ident: &mut <#shared_storage_type as statig::blocking::IntoStateMachine>::Context<'_>
                    ) {
                        match self {
                            #(#call_exit_action_arms),*
                        }
                    }

                    fn superstate(&mut self) -> Option<<#shared_storage_type as statig::blocking::IntoStateMachine>::Superstate<'_>> {
                        match self {
                            #(#superstate_arms),*
                        }
                    }
                }
            )
        }
        Mode::Awaitable => parse_quote!(
            #[allow(unused)]
            impl #impl_generics statig::awaitable::State<#shared_storage_type> for #state_ident #state_generics #where_clause
            {
                #[allow(clippy::manual_async_fn)]
                fn call_handler(
                    &mut self,
                    shared_storage: &mut #shared_storage_type,
                    #event_ident: &<#shared_storage_type as statig::awaitable::IntoStateMachine>::Event<'_>,
                    #context_ident: &mut <#shared_storage_type as statig::awaitable::IntoStateMachine>::Context<'_>
                ) -> impl core::future::Future<Output = statig::Response<Self>> {
                    async move {
                        match self {
                            #(#call_handler_arms),*
                        }
                    }
                }

                #[allow(clippy::manual_async_fn)]
                fn call_entry_action(
                    &mut self,
                    shared_storage: &mut #shared_storage_type,
                    #context_ident: &mut <#shared_storage_type as statig::awaitable::IntoStateMachine>::Context<'_>
                ) -> impl core::future::Future<Output = ()> {
                    async move {
                        match self {
                            #(#call_entry_action_arms),*
                        }
                    }
                }

                #[allow(clippy::manual_async_fn)]
                fn call_exit_action(
                    &mut self,
                    shared_storage: &mut #shared_storage_type,
                    #context_ident: &mut <#shared_storage_type as statig::awaitable::IntoStateMachine>::Context<'_>
                ) -> impl core::future::Future<Output = ()> {
                    async move {
                        match self {
                            #(#call_exit_action_arms),*
                        }
                    }
                }

                #[allow(clippy::manual_async_fn)]
                fn superstate(&mut self) -> Option<<#shared_storage_type as statig::awaitable::IntoStateMachine>::Superstate<'_>> {
                    match self {
                        #(#superstate_arms),*
                    }
                }
            }
        ),
    }
}

fn codegen_superstate(ir: &Ir) -> ItemEnum {
    let superstate_ident = &ir.state_machine.superstate_ident;
    let (superstate_generics, _, _) = &ir.state_machine.superstate_generics.split_for_impl();
    let superstate_derives = &ir.state_machine.superstate_derives;

    let variants: Vec<Variant> = ir
        .superstates
        .values()
        .map(|superstate| superstate.variant.clone())
        .collect();
    let visibility = &ir.state_machine.visibility;

    parse_quote!(
        #[derive(#(#superstate_derives),*)]
        #visibility enum #superstate_ident #superstate_generics {
            #(#variants),*
        }
    )
}

fn codegen_superstate_impl_superstate(ir: &Ir) -> ItemImpl {
    let shared_storage_type = &ir.state_machine.shared_storage_type;

    let (impl_generics, _, where_clause) =
        ir.state_machine.superstate_impl_generics.split_for_impl();
    let superstate_ident = &ir.state_machine.superstate_ident;
    let (_, superstate_generics, _) = &ir.state_machine.superstate_generics.split_for_impl();
    let event_ident = &ir.state_machine.event_ident;
    let context_ident = &ir.state_machine.context_ident;

    let mut call_handler_arms: Vec<Arm> = Vec::new();
    let mut call_entry_action_arms: Vec<Arm> = Vec::new();
    let mut call_exit_action_arms: Vec<Arm> = Vec::new();
    let mut superstate_arms: Vec<Arm> = Vec::new();
    let mut same_state_arms: Vec<Arm> = Vec::new();

    for state in ir.superstates.values() {
        let pat = &state.pat;
        let handler_call = &state.handler_call;
        let entry_action_call = &state.entry_action_call;
        let exit_action_call = &state.exit_action_call;
        let superstate_pat = &state.superstate_pat;

        call_handler_arms.push(parse_quote!(#pat => #handler_call));
        call_entry_action_arms.push(parse_quote!(#pat => #entry_action_call));
        call_exit_action_arms.push(parse_quote!(#pat => #exit_action_call));
        superstate_arms.push(parse_quote!(#pat => #superstate_pat));
    }

    call_handler_arms.push(parse_quote!(_ => statig::Response::Super));
    call_entry_action_arms.push(parse_quote!(_ => {}));
    call_exit_action_arms.push(parse_quote!(_ => {}));
    superstate_arms.push(parse_quote!(_ => None));
    same_state_arms.push(parse_quote!(_ => false));

    match ir.state_machine.mode {
        Mode::Blocking => {
            parse_quote!(
                #[allow(unused)]
                impl #impl_generics statig::blocking::Superstate<#shared_storage_type> for #superstate_ident #superstate_generics #where_clause
                {
                    fn call_handler(
                        &mut self,
                        shared_storage: &mut #shared_storage_type,
                        #event_ident: &<#shared_storage_type as statig::blocking::IntoStateMachine>::Event<'_>,
                        #context_ident: &mut <#shared_storage_type as statig::blocking::IntoStateMachine>::Context<'_>
                    ) -> statig::Response<<#shared_storage_type as statig::blocking::IntoStateMachine>::State> where Self: Sized {
                        match self {
                            #(#call_handler_arms),*
                        }
                    }

                    fn call_entry_action(
                        &mut self,
                        shared_storage: &mut #shared_storage_type,
                        #context_ident: &mut <#shared_storage_type as statig::blocking::IntoStateMachine>::Context<'_>
                    ) {
                        match self {
                            #(#call_entry_action_arms),*
                        }
                    }

                    fn call_exit_action(
                        &mut self,
                        shared_storage: &mut #shared_storage_type,
                        #context_ident: &mut <#shared_storage_type as statig::blocking::IntoStateMachine>::Context<'_>
                    ) {
                        match self {
                            #(#call_exit_action_arms),*
                        }
                    }

                    fn superstate(&mut self) -> Option<<#shared_storage_type as statig::blocking::IntoStateMachine>::Superstate<'_>> {
                        match self {
                            #(#superstate_arms),*
                        }
                    }
                }
            )
        }
        Mode::Awaitable => {
            parse_quote!(
                #[allow(unused)]
                impl #impl_generics statig::awaitable::Superstate<#shared_storage_type> for #superstate_ident #superstate_generics #where_clause
                {
                    #[allow(clippy::manual_async_fn)]
                    fn call_handler(
                        &mut self,
                        shared_storage: &mut #shared_storage_type,
                        #event_ident: &<#shared_storage_type as statig::awaitable::IntoStateMachine>::Event<'_>,
                        #context_ident: &mut <#shared_storage_type as statig::awaitable::IntoStateMachine>::Context<'_>
                    ) -> impl core::future::Future<Output = statig::Response<<#shared_storage_type as statig::awaitable::IntoStateMachine>::State>> {
                        async move {
                            match self {
                                #(#call_handler_arms),*
                            }
                        }
                    }

                    #[allow(clippy::manual_async_fn)]
                    fn call_entry_action(
                        &mut self,
                        shared_storage: &mut #shared_storage_type,
                        #context_ident: &mut <#shared_storage_type as statig::awaitable::IntoStateMachine>::Context<'_>
                    ) -> impl core::future::Future<Output = ()> {
                        async move {
                            match self {
                                #(#call_entry_action_arms),*
                            }
                        }
                    }

                    #[allow(clippy::manual_async_fn)]
                    fn call_exit_action(
                        &mut self,
                        shared_storage: &mut #shared_storage_type,
                        #context_ident: &mut <#shared_storage_type as statig::awaitable::IntoStateMachine>::Context<'_>
                    ) -> impl core::future::Future<Output = ()> {
                        async move {
                            match self {
                                #(#call_exit_action_arms),*
                            }
                        }
                    }

                    #[allow(clippy::manual_async_fn)]
                    fn superstate(&mut self) -> Option<Self> {
                        match self {
                            #(#superstate_arms),*
                        }
                    }
                }
            )
        }
    }
}
