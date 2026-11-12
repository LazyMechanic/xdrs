use darling::util::SpannedValue;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    receivers::{DeriveReceiver, DiscriminantVariantReceiver},
    utils::check_repr_i32,
};

pub fn impl_discriminant_derive(
    mut input: SpannedValue<DeriveReceiver>,
) -> syn::Result<TokenStream> {
    let darling::ast::Data::Enum(data) = &mut input.data else {
        return Err(syn::Error::new(
            input.span(),
            "only enum with discriminant variants are supported",
        ));
    };

    let data = data
        .iter()
        .cloned()
        .map(|v| {
            DiscriminantVariantReceiver::try_from(v)
                .map(DiscriminantVariantReceiver::into_spanned)
        })
        .collect::<Result<Vec<_>, _>>()?;

    check_repr_i32(&input)?;

    let ident = &input.ident;
    let (impl_gen, type_gen, where_gen) = input.generics.split_for_impl();

    let variant_idents = data.iter().map(|v| &v.ident);
    let const_idents =
        variant_idents.clone().map(|v| format_ident!("{}_REPR", v));
    let const_def = const_idents
        .clone()
        .zip(variant_idents.clone())
        .map(|(const_ident, variant_ident)| quote! {
            // SAFETY: If the enumeration specifies a primitive representation,
            // then the discriminant may be reliably accessed via unsafe pointer casting.
            //
            // https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
            #[allow(non_upper_case_globals)]
            const #const_ident: i32 = unsafe { *(&#ident :: #variant_ident as *const #ident as *const i32) };
        });
    let match_branches = const_idents.zip(variant_idents).map(|(const_ident, variant_ident)| quote! {
        x if x == #const_ident => _xdrs::macro_reexport::Option::Some(Self :: #variant_ident)
    });

    Ok(quote! {
        const _: () = {
            extern crate xdrs as _xdrs;
            impl #impl_gen _xdrs::Discriminant for #ident #type_gen #where_gen {
                fn as_discriminant(&self) -> i32 {
                    // SAFETY: If the enumeration specifies a primitive representation,
                    // then the discriminant may be reliably accessed via unsafe pointer casting.
                    //
                    // https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
                    unsafe { *(self as *const Self as *const i32) }
                }

                fn from_discriminant(discriminant: i32) -> _xdrs::macro_reexport::Option<Self> {
                    #( #const_def )*
                    match discriminant {
                        #( #match_branches, )*
                        _ => _xdrs::macro_reexport::Option::None,
                    }
                }
            }
        };
    })
}
