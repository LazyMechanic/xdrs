use darling::util::SpannedValue;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Pat, parse_quote};

use crate::{
    receivers::{
        DeriveReceiver, DiscriminantVariantReceiver, FieldReceiver,
        VariantReceiver,
    },
    utils::{
        check_enum_discriminant_attrs, check_enum_discriminant_has_variants,
        check_enum_union_attrs, check_enum_union_has_variants,
        check_no_generics, check_repr_i32, check_struct_attrs,
        check_struct_is_struct,
    },
};

pub fn impl_encode_derive(
    input: SpannedValue<DeriveReceiver>,
) -> syn::Result<TokenStream> {
    match &input.data {
        darling::ast::Data::Struct(data) => impl_struct(&input, data),
        darling::ast::Data::Enum(data) => {
            if input.switch_attr.is_some() {
                impl_union(&input, data)
            } else {
                let data = data
                    .iter()
                    .cloned()
                    .map(|v| {
                        DiscriminantVariantReceiver::try_from(v)
                            .map(DiscriminantVariantReceiver::into_spanned)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                impl_enum(&input, &data)
            }
        }
    }
}

fn impl_struct(
    input: &SpannedValue<DeriveReceiver>,
    data: &darling::ast::Fields<SpannedValue<FieldReceiver>>,
) -> syn::Result<TokenStream> {
    validate_impl_struct(input, data)?;

    let fields = data
        .fields
        .iter()
        .map(|f| {
            f.ident.as_ref().ok_or_else(|| {
                syn::Error::new(f.span(), "tuple like structs not supported")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let ident = &input.ident;
    let (impl_gen, type_gen, where_gen) = input.generics.split_for_impl();

    let out = quote! {
        const _: () = {
            extern crate xdrs as _xdrs;
            impl #impl_gen _xdrs::Encode for #ident #type_gen #where_gen {
                fn encode<__E: _xdrs::Encoder>(&self, encoder: &mut __E) -> _xdrs::macro_reexport::Result<(), __E::Error> {
                    #(
                        _xdrs::Encode::encode(&self.#fields, encoder)?;
                    )*
                    Ok(())
                }
            }
        };
    };
    Ok(out)
}

fn impl_enum(
    input: &SpannedValue<DeriveReceiver>,
    data: &[SpannedValue<DiscriminantVariantReceiver>],
) -> syn::Result<TokenStream> {
    validate_impl_enum(input, data)?;

    let ident = &input.ident;
    let (impl_gen, type_gen, where_gen) = input.generics.split_for_impl();

    let out = quote! {
        const _: () = {
            extern crate xdrs as _xdrs;
            impl #impl_gen _xdrs::Encode for #ident #type_gen #where_gen {
                fn encode<__E: _xdrs::Encoder>(&self, encoder: &mut __E) -> _xdrs::macro_reexport::Result<(), __E::Error> {
                    // SAFETY: If the enumeration specifies a primitive representation,
                    // then the discriminant may be reliably accessed via unsafe pointer casting.
                    //
                    // https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
                    let this_i32 = unsafe { *(self as *const Self as *const i32) };
                    _xdrs::Encode::encode(&this_i32, encoder)
                }
            }
        };
    };
    Ok(out)
}

fn impl_union(
    input: &SpannedValue<DeriveReceiver>,
    data: &[SpannedValue<VariantReceiver>],
) -> syn::Result<TokenStream> {
    validate_impl_union(input, data)?;

    pub fn get_variant_pat(
        v: &SpannedValue<VariantReceiver>,
    ) -> syn::Result<Pat> {
        let ident = &v.ident;
        if v.fields.is_unit() {
            Ok(parse_quote!( Self::#ident ))
        } else if v.fields.is_tuple() && v.fields.fields.len() == 1 {
            Ok(parse_quote!( Self::#ident(v) ))
        } else {
            Err(syn::Error::new(
                v.span(),
                "tuple and unit variants supported only",
            ))
        }
    }

    fn variant_encode_impl(
        v: &SpannedValue<VariantReceiver>,
        disc_value: TokenStream,
    ) -> syn::Result<TokenStream> {
        if v.fields.is_unit() {
            Ok(quote! {
                // Discriminant
                _xdrs::Encode::encode(&#disc_value, encoder)?;
                // No value
                Ok(())
            })
        } else if v.fields.is_tuple() && v.fields.fields.len() == 1 {
            Ok(quote! {
                // Discriminant
                _xdrs::Encode::encode(&#disc_value, encoder)?;
                // Value
                _xdrs::Encode::encode(v, encoder)?;
                Ok(())
            })
        } else {
            Err(syn::Error::new(
                v.span(),
                "tuple and unit variants supported only",
            ))
        }
    }

    let match_branches = data
        .iter()
        .map(|v| {
            let var_pat = get_variant_pat(v)?;

            // #[xdrs(discriminant = "...")]
            if let Some(attr) = &v.discriminant_attr {
                let disc_value = {
                    let disc_variant = &***attr;
                    quote! { #disc_variant }
                };

                let encode_impl = variant_encode_impl(v, disc_value)?;
                return Ok(quote! {
                    #var_pat => { #encode_impl }
                });
            }

            // #[xdrs(default)]
            if v.default_attr.is_present() {
                return Ok(quote! {
                    #var_pat => { _xdrs::macro_reexport::Result::Err(<__E::Error as _xdrs::encode::EncodeError>::invalid_variant("default branch lacks an explicit discriminant value")) }
                });
            }

            Err(syn::Error::new(
                v.span(),
                "missing `#[xdrs(discriminant = \"...\")]` or `#[xdrs(default)]` attribute",
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let ident = &input.ident;
    let (impl_gen, type_gen, where_gen) = input.generics.split_for_impl();

    let out = quote! {
        const _: () = {
            extern crate xdrs as _xdrs;
            impl #impl_gen _xdrs::Encode for #ident #type_gen #where_gen {
                fn encode<__E: _xdrs::Encoder>(&self, encoder: &mut __E) -> _xdrs::macro_reexport::Result<(), __E::Error> {
                    match self {
                        #(#match_branches),*
                    }
                }
            }
        };
    };
    Ok(out)
}

fn validate_impl_struct(
    input: &SpannedValue<DeriveReceiver>,
    data: &darling::ast::Fields<SpannedValue<FieldReceiver>>,
) -> syn::Result<()> {
    check_struct_is_struct(input, data)?;
    check_struct_attrs(input, data)?;

    Ok(())
}

fn validate_impl_enum(
    input: &SpannedValue<DeriveReceiver>,
    data: &[SpannedValue<DiscriminantVariantReceiver>],
) -> syn::Result<()> {
    check_repr_i32(&input)?;
    check_no_generics(&input.generics)?;
    check_enum_discriminant_has_variants(input, data)?;
    check_enum_discriminant_attrs(input, data)?;

    Ok(())
}

fn validate_impl_union(
    input: &SpannedValue<DeriveReceiver>,
    data: &[SpannedValue<VariantReceiver>],
) -> syn::Result<()> {
    check_enum_union_has_variants(input, data)?;
    check_enum_union_attrs(input, data)?;

    Ok(())
}
