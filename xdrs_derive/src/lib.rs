mod encode;

use darling::{FromDeriveInput, util::SpannedValue};
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use crate::encode::{EncodeReceiver, impl_encode_derive};

#[proc_macro_derive(Encode, attributes(xdrs))]
pub fn encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match SpannedValue::<EncodeReceiver>::from_derive_input(&input) {
        Ok(parsed) => match impl_encode_derive(parsed) {
            Ok(expanded) => expanded.into(),
            Err(err) => err.to_compile_error().into(),
        },
        Err(err) => err.write_errors().into(),
    }
}
