extern crate proc_macro;

mod elusiv_account;
mod elusiv_hash_compute_units;
mod repeat;
mod parse_tokens;
mod program_id;
mod utils;

use syn::{ parse_macro_input, DeriveInput };
use elusiv_account::impl_elusiv_account;
use elusiv_hash_compute_units::impl_elusiv_hash_compute_units;
use repeat::impl_repeat;
use parse_tokens::impl_parse_tokens;
use program_id::impl_program_id;

/// Just-in-time mutable-byte-slice-backed serialization account
/// - every field is represented by a `&mut [u8]`
/// - every field has a setter (serialization) and getter (deserialization) function
/// - to prevent the getter-setter creation use the attribute: `pub_non_lazy`
#[proc_macro_attribute]
pub fn elusiv_account(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_elusiv_account(&ast, args.into()).into()
}

/// Creates a struct `Name` that implements `elusiv_computation::PartialComputation`
/// - `elusiv_hash_compute_units!(<name>, <NUMBER_OF_HASHES>)`
#[proc_macro]
pub fn elusiv_hash_compute_units(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_elusiv_hash_compute_units(input.into()).into()
}

/// Repeates an expression count times
/// 
/// ### Usage
/// - `repeat!({<<expr>>}, <<count>>)`
/// - use `_index` inside of `<<expr>>` to get the current index of the loop
#[proc_macro]
pub fn repeat(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_repeat(input.into()).into()
}

/// Parses Token.toml
#[proc_macro]
pub fn elusiv_tokens(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_parse_tokens().into()
}

#[proc_macro]
pub fn program_id(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_program_id().into()
}