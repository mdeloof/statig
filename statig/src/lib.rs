#![cfg_attr(not(doctest), doc = include_str!(concat!("../", core::env!("CARGO_PKG_README"))))]
#![no_std]
#![allow(incomplete_features)]

mod outcome;
mod state_or_superstate;

/// Macro for deriving the state and superstate enum.
///
/// By parsing the underlying `impl` block and searching for methods with the
/// `state`, `superstate` or `action` attribute, the `state_machine` macro can
/// derive the state and superstate enums. By default these will be given the
/// names '`State`' and '`Superstate`'. Next to that the macro will also
/// implement the [`State`](crate::blocking::State) trait for the state enum and the
/// [`Superstate`](crate::blocking::Superstate) trait for the superstate enum.
///
/// To override the default configuration you can use the following attributes.
///
/// - `#[state_machine(state(name = "CustomStateName"))]`
///
///   Set the name of the state enum to a custom name.
///
///   _Default_: `State`
///
///   <br/>
///
/// - `#[state_machine(superstate(name = "CustomSuperstateName"))]`
///
///   Set the name of the superstate enum to a custom name.
///
///   _Default_: `Superstate`
///
///   <br/>
///
/// - `#[state_machine(state(derive(SomeTrait, AnotherTrait)))]`
///
///   Apply the derive macro with the passed traits to the state enum.
///
///   _Default_: `()`
///
///   <br/>
///
/// - `#[state_machine(superstate(derive(SomeTrait, AnotherTrait)))]`
///
///   Apply the derive macro with the passed traits to the superstate enum.
///
///   _Default_: `()`
///
///   <br/>
#[cfg(feature = "macro")]
#[cfg_attr(docsrs, doc(cfg(feature = "macro")))]
pub use statig_macro::state_machine;

/// Attribute for tagging a state.
///
/// This macro does nothing on its own but is detected by the `state_machine`
/// macro when added to a method.
///
/// It accepts the following attributes:
///
/// - `#[state(name = "CustomStateName")]`
///
///   Set the name of the variant that will be part of the state enum.
///
///   <br/>
///
/// - `#[state(superstate = "superstate_name")]`
///
///   Set the superstate of the state.
///
///   <br/>
///
/// - `#[state(entry_action = "entry_action_name")]`
///
///   Set the entry action of the state.
///
///   <br/>
///
/// - `#[state(exit_action = "exit_action_name")]`
///
///   Set the exit action of the state.
///
///   <br/>
///
/// - `#[state(local_storage("field_name_a: FieldTypeA", "field_name_b: FieldTypeB"))]`
///
///   Add local storage to this state. These will be added as fields to the enum variant.
///
///   <br/>
#[cfg(feature = "macro")]
#[cfg_attr(docsrs, doc(cfg(feature = "macro")))]
pub use statig_macro::state;

/// Attribute for tagging a superstate.
///
/// This macro does nothing on its own but is detected by the `state_machine`
/// macro when added to a method.
///
/// It accepts the following attributes:
///
/// - `#[superstate(name = "CustomSuperstateName")]`
///
///   Set the name of the variant that will be part of the state enum.
///
///   <br/>
///
/// - `#[superstate(superstate = "superstate_name")]`
///
///   Set the superstate of the superstate.
///
///   <br/>
///
/// - `#[superstate(entry_action = "entry_action_name")]`
///
///   Set the entry action of the superstate.
///
///   <br/>
///
/// - `#[superstate(exit_action = "exit_action_name")]`
///
///   Set the exit action of the superstate.
///
///   <br/>
///
/// - `#[superstate(local_storage("field_name_a: &'a mut FieldTypeA"))]`
///
///   Add local storage to this superstate. These will be added as fields to
///   the enum variant. It is crucial to understand that superstates never own
///   their data. Instead it is always borrowed from the underlying state or
///   superstate. This means the fields should be references with an
///   associated lifetime `'a`.
///
///   <br/>
#[cfg(feature = "macro")]
#[cfg_attr(docsrs, doc(cfg(feature = "macro")))]
pub use statig_macro::superstate;

/// Attribute for tagging an action.
///
/// This macro does nothing on its own but is detected by the `state_machine`
/// macro when added to a method.
#[cfg(feature = "macro")]
#[cfg_attr(docsrs, doc(cfg(feature = "macro")))]
pub use statig_macro::action;

/// Prelude containing the necessary imports for use with macro.
pub mod prelude {
    #![allow(ambiguous_glob_reexports)]
    #![allow(unused_imports)]

    #[cfg(feature = "async")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
    pub use crate::awaitable::{IntoStateMachineExt as _, StateExt as _, *};
    pub use crate::blocking::{IntoStateMachineExt as _, StateExt as _, *};
    pub use crate::Outcome::{self, *};
    pub use crate::StateOrSuperstate;
    #[cfg(feature = "macro")]
    #[cfg_attr(docsrs, doc(cfg(feature = "macro")))]
    pub use statig_macro::state_machine;
}

pub mod blocking;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod awaitable;

pub use outcome::*;
pub use state_or_superstate::*;
