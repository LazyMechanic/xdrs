use darling::util::SpannedValue;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    GenericParam, Lifetime, LifetimeParam, Token,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
};

use crate::{
    receivers::{
        DeriveReceiver, DiscriminantVariantReceiver, FieldReceiver,
        VariantReceiver,
    },
    utils::{
        check_enum_discriminant_attrs, check_enum_union_attrs,
        check_no_generics, check_repr_i32, check_struct_attrs,
        check_struct_is_struct,
    },
};

pub fn impl_decode_derive(
    mut input: SpannedValue<DeriveReceiver>,
) -> syn::Result<TokenStream> {
    if let darling::ast::Data::Enum(data) = &mut input.data {
        data.sort_by_key(|d| d.default_attr.is_present());
    }

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

    let decode_lt: Lifetime = parse_quote! { '__d };
    let impl_gen = {
        let mut impl_gen: ImplGenerics = parse_quote! { #impl_gen };
        let existing_lifetimes: Vec<&Lifetime> = input
            .generics
            .lifetimes()
            .map(|lt_param| &lt_param.lifetime)
            .collect();

        let decode_lt_def: LifetimeParam = if existing_lifetimes.is_empty() {
            parse_quote! { #decode_lt }
        } else {
            parse_quote! { #decode_lt : #(#existing_lifetimes)+* }
        };

        impl_gen
            .params
            .insert(0, GenericParam::Lifetime(decode_lt_def));
        impl_gen
    };

    Ok(quote! {
        const _: () = {
            extern crate xdrs as _xdrs;
            impl #impl_gen _xdrs::Decode<#decode_lt> for #ident #type_gen #where_gen {
                fn decode<__D: _xdrs::Decoder<#decode_lt>>(decoder: &mut __D) -> _xdrs::macro_reexport::Result<Self, __D::Error> {
                    Ok(Self {
                        #( #fields: _xdrs::Decode::decode(decoder)?, )*
                    })
                }
            }
        };
    })
}

fn impl_enum(
    input: &SpannedValue<DeriveReceiver>,
    data: &[SpannedValue<DiscriminantVariantReceiver>],
) -> syn::Result<TokenStream> {
    validate_impl_enum(input, data)?;

    let ident = &input.ident;
    let (impl_gen, type_gen, where_gen) = input.generics.split_for_impl();

    let decode_lt: Lifetime = parse_quote! { '__d };
    let impl_gen = {
        let mut impl_gen: ImplGenerics = parse_quote! { #impl_gen };
        let existing_lifetimes: Vec<&Lifetime> = input
            .generics
            .lifetimes()
            .map(|lt_param| &lt_param.lifetime)
            .collect();

        let decode_lt_def: LifetimeParam = if existing_lifetimes.is_empty() {
            parse_quote! { #decode_lt }
        } else {
            parse_quote! { #decode_lt : #(#existing_lifetimes)+* }
        };

        impl_gen
            .params
            .insert(0, GenericParam::Lifetime(decode_lt_def));
        impl_gen
    };

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
        x if x == #const_ident => _xdrs::macro_reexport::Result::Ok(Self :: #variant_ident)
    });

    Ok(quote! {
        const _: () = {
            extern crate xdrs as _xdrs;
            impl #impl_gen _xdrs::Decode<#decode_lt> for #ident #type_gen #where_gen {
                fn decode<__D: _xdrs::Decoder<#decode_lt>>(decoder: &mut __D) -> _xdrs::macro_reexport::Result<Self, __D::Error> {
                    #(#const_def)*
                    let v: i32 = _xdrs::Decode::decode(decoder)?;
                    match v {
                        #( #match_branches, )*
                        unknown => _xdrs::macro_reexport::Result::Err(<__D::Error as _xdrs::decode::DecodeError>::unknown_variant(unknown)),
                    }
                }
            }
        };
    })
}

fn impl_union(
    input: &SpannedValue<DeriveReceiver>,
    data: &[SpannedValue<VariantReceiver>],
) -> syn::Result<TokenStream> {
    validate_impl_union(input, data)?;

    // SAFETY: impl_union() is called after the `switch` attribute has been checked.
    let switch = input
        .switch_attr
        .as_ref()
        .expect("missing `#[xdrs(switch = \"...\" )]` attribute");
    let discriminant_ty = &***switch;

    let ident = &input.ident;
    let (impl_gen, type_gen, where_gen) = input.generics.split_for_impl();

    let decode_lt: Lifetime = parse_quote! { '__d };
    let impl_gen = {
        let mut impl_gen: ImplGenerics = parse_quote! { #impl_gen };
        let existing_lifetimes: Vec<&Lifetime> = input
            .generics
            .lifetimes()
            .map(|lt_param| &lt_param.lifetime)
            .collect();

        let decode_lt_def: LifetimeParam = if existing_lifetimes.is_empty() {
            parse_quote! { #decode_lt }
        } else {
            parse_quote! { #decode_lt : #(#existing_lifetimes)+* }
        };

        impl_gen
            .params
            .insert(0, GenericParam::Lifetime(decode_lt_def));
        impl_gen
    };

    let mut default_present = false;
    let match_branches = data.iter().map(|v| {
        let variant_ident = &v.ident;
        // #[xdrs(discriminant = "...")]
        if let Some(attr) = &v.discriminant_attr {
            let discriminant_expr = &***attr;

            // Decode void (noop)
            if v.fields.is_unit() {
                return Ok(quote! {
                    x if x == (#discriminant_expr as i32) => {
                        _xdrs::macro_reexport::Result::Ok(Self :: #variant_ident)
                    },
                });
            }

            // Decode type
            if v.fields.fields.first().is_some() {
                return Ok(quote! {
                    x if x == (#discriminant_expr as i32) => {
                        _xdrs::macro_reexport::Result::Ok(Self :: #variant_ident ( _xdrs::Decode::decode(decoder)? ))
                    }
                });
            }
        }

        // #[xdrs(default)]
        if v.default_attr.is_present() {
            default_present = true;

            // Decode void (noop)
            if v.fields.is_unit() {
                return Ok(quote! {
                    _ => {
                        _xdrs::macro_reexport::Result::Ok(Self :: #variant_ident)
                    }
                });
            }

            // Decode type
            if v.fields.fields.first().is_some() {
                return Ok(quote! {
                    _ => {
                        _xdrs::macro_reexport::Result::Ok(Self :: #variant_ident ( _xdrs::Decode::decode(decoder)? ))
                    }
                });
            }
        }

        Err(syn::Error::new(
            v.span(),
            "unsupported syntax",
        ))
    }).collect::<Result<Vec<_>, _>>()?;

    let match_branches: Box<dyn Iterator<Item = _>> = if default_present {
        Box::new(match_branches.into_iter())
    } else {
        Box::new(match_branches.into_iter().chain(std::iter::once(quote! {
            unknown => {
                _xdrs::macro_reexport::Result::Err(<__D::Error as _xdrs::decode::DecodeError>::unknown_variant(unknown))
            }
        })))
    };

    Ok(quote! {
        const _: () = {
            extern crate xdrs as _xdrs;
            impl #impl_gen _xdrs::Decode<#decode_lt> for #ident #type_gen #where_gen {
                fn decode<__D: _xdrs::Decoder<#decode_lt>>(decoder: &mut __D) -> _xdrs::macro_reexport::Result<Self, __D::Error> {
                    let v: i32 = _xdrs::Decode::decode(decoder)?;
                    match v {
                        #( #match_branches, )*
                    }
                }
            }
        };
    })
}

#[derive(Default)]
struct ImplGenerics {
    pub lt_token: Option<Token![<]>,
    pub params: Punctuated<GenericParam, Token![,]>,
    pub gt_token: Option<Token![>]>,
}

impl Parse for ImplGenerics {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if !input.peek(Token![<]) {
            return Ok(ImplGenerics::default());
        }

        let lt_token = input.parse()?;

        let mut params = Punctuated::new();
        while !input.peek(Token![>]) {
            let param: GenericParam = input.parse()?;
            params.push_value(param);
            if input.peek(Token![>]) {
                break;
            }
            let punct: Token![,] = input.parse()?;
            params.push_punct(punct);
        }

        let gt_token = input.parse()?;
        Ok(Self {
            lt_token,
            params,
            gt_token,
        })
    }
}

impl ToTokens for ImplGenerics {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.params.is_empty() {
            return;
        }

        self.lt_token.unwrap_or_default().to_tokens(tokens);
        self.params.to_tokens(tokens);
        self.gt_token.unwrap_or_default().to_tokens(tokens);
    }
}

fn validate_impl_struct(
    input: &SpannedValue<DeriveReceiver>,
    data: &darling::ast::Fields<SpannedValue<FieldReceiver>>,
) -> syn::Result<()> {
    check_struct_is_struct(data)?;
    check_struct_attrs(input, data)?;

    Ok(())
}

fn validate_impl_enum(
    input: &SpannedValue<DeriveReceiver>,
    data: &[SpannedValue<DiscriminantVariantReceiver>],
) -> syn::Result<()> {
    check_repr_i32(&input)?;
    check_no_generics(&input.generics)?;
    check_enum_discriminant_attrs(input, data)?;

    Ok(())
}

fn validate_impl_union(
    input: &SpannedValue<DeriveReceiver>,
    data: &[SpannedValue<VariantReceiver>],
) -> syn::Result<()> {
    check_enum_union_attrs(input, data)?;

    Ok(())
}
