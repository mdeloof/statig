use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Arm, ItemEnum, ItemFn, ItemImpl, Variant};

use crate::lower::{Ir, Mode};

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
    let object_type = &ir.state_machine.shared_storage_type;
    let event_type = &ir.state_machine.event_type;
    let context_type = &ir.state_machine.context_type;
    let state_type = &ir.state_machine.state_type;
    let superstate_type = &ir.state_machine.superstate_type;

    let initial_state = &ir.state_machine.initial_state;

    let mode = match ir.state_machine.mode {
        Mode::Blocking => quote!(blocking),
        Mode::Awaitable => quote!(awaitable),
    };

    let on_transition = match &ir.state_machine.on_transition {
        None => quote!(),
        Some(on_transition) => quote!(
            const ON_TRANSITION: fn(&mut Self, &Self::State, &Self::State) = #on_transition;
        ),
    };

    let on_dispatch = match &ir.state_machine.on_dispatch {
        None => quote!(),
        Some(on_dispatch) => quote!(
            const ON_DISPATCH: fn(&mut Self, StateOrSuperstate<'_, '_, Self>, &Self::Event<'_>) = #on_dispatch;
        ),
    };

    parse_quote!(
        impl statig::#mode::IntoStateMachine for #object_type {
            type Event<'a> = #event_type;
            type Context<'a> = #context_type;
            type State = #state_type;
            type Superstate<'a> = #superstate_type;
            const INITIAL: #state_type = #initial_state;

            #on_transition

            #on_dispatch
        }
    )
}

fn codegen_state(ir: &Ir) -> ItemEnum {
    let state_type = &ir.state_machine.state_type;
    let state_derives = &ir.state_machine.state_derives;
    let variants: Vec<Variant> = ir
        .states
        .values()
        .map(|state| state.variant.clone())
        .collect();
    let visibility = &ir.state_machine.visibility;

    parse_quote!(
        #[derive(#(#state_derives),*)]
        # visibility enum #state_type {
            #(#variants),*
        }
    )
}

fn codegen_state_impl(ir: &Ir) -> ItemImpl {
    let state_type = &ir.state_machine.state_type;

    let constructors: Vec<ItemFn> = ir
        .states
        .values()
        .map(|state| &state.constructor)
        .cloned()
        .collect();

    parse_quote!(
        impl #state_type {
            #(#constructors)*
        }
    )
}

fn codegen_state_impl_state(ir: &Ir) -> ItemImpl {
    let object_type = &ir.state_machine.shared_storage_type;
    let state_type = &ir.state_machine.state_type;
    let event_ident = &ir.state_machine.event_ident;
    let context_ident = &ir.state_machine.context_ident;

    let mut constructors: Vec<ItemFn> = Vec::new();
    let mut call_handler_arms: Vec<Arm> = Vec::new();
    let mut call_entry_action_arms: Vec<Arm> = Vec::new();
    let mut call_exit_action_arms: Vec<Arm> = Vec::new();
    let mut superstate_arms: Vec<Arm> = Vec::new();
    let mut same_state_arms: Vec<Arm> = Vec::new();

    let (mode, asyncness) = match ir.state_machine.mode {
        Mode::Blocking => (quote!(blocking), quote!()),
        Mode::Awaitable => (quote!(awaitable), quote!(async)),
    };

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

    parse_quote!(
        #[allow(unused)]
        impl statig::#mode::State<#object_type> for #state_type {
            #asyncness fn call_handler(&mut self, shared_storage: &mut #object_type, #event_ident: &<#object_type as statig::#mode::IntoStateMachine>::Event<'_>, #context_ident: &mut <#object_type as statig::#mode::IntoStateMachine>::Context<'_>) -> statig::Response<Self> where Self: Sized {
                match self {
                    #(#call_handler_arms),*
                }
            }

            #asyncness fn call_entry_action(&mut self, shared_storage: &mut #object_type, #context_ident: &mut <#object_type as statig::#mode::IntoStateMachine>::Context<'_>) {
                match self {
                    #(#call_entry_action_arms),*
                }
            }

            #asyncness fn call_exit_action(&mut self, shared_storage: &mut #object_type, #context_ident: &mut <#object_type as statig::#mode::IntoStateMachine>::Context<'_>) {
                match self {
                    #(#call_exit_action_arms),*
                }
            }

            fn superstate(&mut self) -> Option<<#object_type as statig::#mode::IntoStateMachine>::Superstate<'_>> {
                match self {
                    #(#superstate_arms),*
                }
            }
        }
    )
}

fn codegen_superstate(ir: &Ir) -> ItemEnum {
    let superstate_type = &ir.state_machine.superstate_type;
    let superstate_derives = &ir.state_machine.superstate_derives;
    let variants: Vec<Variant> = ir
        .superstates
        .values()
        .map(|superstate| superstate.variant.clone())
        .collect();
    let visibility = &ir.state_machine.visibility;

    parse_quote!(
        #[derive(#(#superstate_derives),*)]
        #visibility enum #superstate_type {
            #(#variants),*
        }
    )
}

fn codegen_superstate_impl_superstate(ir: &Ir) -> ItemImpl {
    let shared_storage_type = &ir.state_machine.shared_storage_type;
    let superstate_type = &ir.state_machine.superstate_type;
    let event_ident = &ir.state_machine.event_ident;
    let context_ident = &ir.state_machine.context_ident;

    let mut call_handler_arms: Vec<Arm> = Vec::new();
    let mut call_entry_action_arms: Vec<Arm> = Vec::new();
    let mut call_exit_action_arms: Vec<Arm> = Vec::new();
    let mut superstate_arms: Vec<Arm> = Vec::new();
    let mut same_state_arms: Vec<Arm> = Vec::new();

    let (mode, asyncness) = match ir.state_machine.mode {
        Mode::Blocking => (quote!(blocking), quote!()),
        Mode::Awaitable => (quote!(awaitable), quote!(async)),
    };

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

    parse_quote!(
        #[allow(unused)]
        impl<'a> statig::#mode::Superstate<#shared_storage_type> for #superstate_type
        where
            Self: 'a,
        {
            #asyncness fn call_handler(
                &mut self,
                shared_storage: &mut #shared_storage_type,
                #event_ident: &<#shared_storage_type as statig::#mode::IntoStateMachine>::Event<'_>,
                #context_ident: &mut <#shared_storage_type as statig::#mode::IntoStateMachine>::Context<'_>
            ) -> statig::Response<<#shared_storage_type as statig::#mode::IntoStateMachine>::State> where Self: Sized {
                match self {
                    #(#call_handler_arms),*
                }
            }

            #asyncness fn call_entry_action(
                &mut self,
                shared_storage: &mut #shared_storage_type,
                #context_ident: &mut <#shared_storage_type as statig::#mode::IntoStateMachine>::Context<'_>
            ) {
                match self {
                    #(#call_entry_action_arms),*
                }
            }

            #asyncness fn call_exit_action(
                &mut self,
                shared_storage: &mut #shared_storage_type,
                #context_ident: &mut <#shared_storage_type as statig::#mode::IntoStateMachine>::Context<'_>
            ) {
                match self {
                    #(#call_exit_action_arms),*
                }
            }

            fn superstate(&mut self) -> Option<<#shared_storage_type as statig::#mode::IntoStateMachine>::Superstate<'_>> {
                match self {
                    #(#superstate_arms),*
                }
            }
        }
    )
}
