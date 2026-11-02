use std::ops::Deref;

use darling::{
    FromDeriveInput, FromField, FromMeta, FromVariant, util::SpannedValue,
};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(FromDeriveInput)]
#[darling(attributes(xdrs), forward_attrs(repr))]
pub struct EncodeReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<
        SpannedValue<VariantReceiver>,
        SpannedValue<FieldReceiver>,
    >,
    attrs: Vec<syn::Attribute>,

    // #[xdrs(switch = "path::to::Enum")] over enum union-like
    #[darling(default)]
    switch: Option<SpannedValue<XdrsSwitchAttr>>,
}

/// `#[xdrs(switch = "path::to::Enum")]`
#[derive(Debug, Clone)]
struct XdrsSwitchAttr(syn::Path);

impl Deref for XdrsSwitchAttr {
    type Target = syn::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromMeta for XdrsSwitchAttr {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self(syn::Path::from_string(value)?))
    }
}

/// Struct fields
/// ```text
/// a: i32,
/// b: String,
/// ...
/// ```
#[derive(FromField)]
#[darling(attributes(xdrs))]
struct FieldReceiver {
    ident: Option<syn::Ident>,
}

/// Enum variants
/// ```text
/// VariantA(StructA),
/// VariantB = 123,
/// ...
/// ```
#[derive(FromVariant)]
#[darling(attributes(xdrs))]
struct VariantReceiver {
    ident: syn::Ident,
    fields: darling::ast::Fields<SpannedValue<VariantFieldReceiver>>,

    // #[xdrs(default)]
    #[darling(default)]
    default: darling::util::Flag,
    // #[xdrs(discriminant = "VariantA")]
    #[darling(default, rename = "discriminant")]
    discriminant_attr: Option<SpannedValue<XdrsDiscriminantAttr>>,
}

/// `#[xdrs(discriminant = "VariantA")]`
#[derive(Debug, Clone)]
struct XdrsDiscriminantAttr(syn::Ident);

impl FromMeta for XdrsDiscriminantAttr {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self(syn::Ident::from_string(value)?))
    }
}

#[derive(FromField)]
struct VariantFieldReceiver {
    ty: syn::Type,
}

pub fn impl_encode_derive(
    input: SpannedValue<EncodeReceiver>,
) -> syn::Result<TokenStream> {
    match &input.data {
        darling::ast::Data::Struct(data) => impl_struct(&input, data),
        darling::ast::Data::Enum(data) => {
            if input.switch.is_some() {
                impl_union(&input, data)
            } else {
                impl_enum(&input, data)
            }
        }
    }
}

fn impl_struct(
    input: &SpannedValue<EncodeReceiver>,
    data: &darling::ast::Fields<SpannedValue<FieldReceiver>>,
) -> syn::Result<TokenStream> {
    if !data.style.is_struct() {
        return Err(syn::Error::new(
            input.span(),
            "only structs with named fields are supported",
        ));
    }

    if let Some(attr) = &input.switch {
        return Err(syn::Error::new(
            attr.span(),
            "unsupported attribute in struct",
        ));
    }

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
                fn encode<__W: _xdrs::reexport::Write>(&self, out: &mut __W) -> _xdrs::reexport::Result<usize, _xdrs::Error> {
                    let mut write = 0;
                    #(
                        write += _xdrs::ResultExt::at_field(_xdrs::Encode::encode(&self.#fields, out), _xdrs::reexport::stringify!(#fields))?;
                    )*
                    Ok(write)
                }
            }
        };
    };
    Ok(out)
}

fn impl_enum(
    input: &SpannedValue<EncodeReceiver>,
    _data: &[SpannedValue<VariantReceiver>],
) -> syn::Result<TokenStream> {
    check_repr_i32(&input)?;

    let ident = &input.ident;
    let (impl_gen, type_gen, where_gen) = input.generics.split_for_impl();

    let out = quote! {
        const _: () = {
            extern crate xdrs as _xdrs;
            impl #impl_gen #ident #type_gen #where_gen {
                pub fn as_i32(&self) -> i32 {
                    // SAFETY: If the enumeration specifies a primitive representation,
                    // then the discriminant may be reliably accessed via unsafe pointer casting.
                    // For an enum, it is mandatory to specify `#[repr(i32)]` so
                    // that `Encode` can be implemented (via derive)
                    //
                    // https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
                    unsafe { *(self as *const Self as *const i32) }
                }
            }

            impl #impl_gen _xdrs::Encode for #ident #type_gen #where_gen {
                fn encode<__W: _xdrs::reexport::Write>(&self, out: &mut __W) -> _xdrs::reexport::Result<usize, _xdrs::Error> {
                    _xdrs::Encode::encode(&self.as_i32(), out)
                }
            }
        };
    };
    Ok(out)
}

fn impl_union(
    input: &SpannedValue<EncodeReceiver>,
    data: &[SpannedValue<VariantReceiver>],
) -> syn::Result<TokenStream> {
    // SAFETY: impl_union() is called after the `switch` attribute has been checked.
    let switch = input
        .switch
        .as_ref()
        .expect("missing `#[xdrs(switch = \"...\" )]` attribute");
    let discriminant_ty = &switch.0;

    let get_variant_pat = |v: &SpannedValue<VariantReceiver>| {
        let ident = &v.ident;
        if v.fields.is_unit() {
            Ok(quote!( Self::#ident ))
        } else if v.fields.is_tuple() && v.fields.fields.len() == 1 {
            Ok(quote!( #[allow(unused_variables)] Self::#ident(v) ))
        } else {
            Err(syn::Error::new(
                v.span(),
                "tuple and unit variants supported only",
            ))
        }
    };

    let mut default_present = false;
    let variant_to_discriminant = data
        .iter()
        .map(|v| {
            let var_pat = get_variant_pat(v)?;
            match (&v.discriminant_attr, &v.default) {
                // #[xdrs(discriminant = "...")]
                (Some(discriminant_attr), default_attr)
                if !default_attr.is_present() => {
                    let disc_variant = &discriminant_attr.0;
                    Ok(quote!( #var_pat => _xdrs::reexport::Option::Some(#discriminant_ty::#disc_variant) ))
                }
                // #[xdrs(default)]
                (None, default_attr) if default_attr.is_present() => {
                    // If default already present then throw error
                    if default_present {
                        return Err(syn::Error::new(default_attr.span(), "only one variant can have the #[xdrs(default)] attribute"))
                    }
                    default_present = true;
                    Ok(quote!( #var_pat => _xdrs::reexport::Option::None ))
                }
                _ => Err(syn::Error::new(v.span(), "exactly one of the following attributes must be specified: `#[xdrs(discriminant = \"...\")]` or `#[xdrs(default)]`")),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let variant_to_encode = data
        .iter()
        .map(|v| {
            let var_pat = get_variant_pat(v)?;
            if v.fields.is_unit() {
                Ok(quote!( #var_pat => _xdrs::Encode::encode(&_xdrs::Void, out) ))
            } else if v.fields.is_tuple() && v.fields.fields.len() == 1 {
                Ok(quote!( #var_pat => _xdrs::Encode::encode(v, out) ))
            } else {
                Err(syn::Error::new(v.span(), "tuple and unit variants supported only"))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let ident = &input.ident;
    let (impl_gen, type_gen, where_gen) = input.generics.split_for_impl();

    let out = quote! {
        const _: () = {
            extern crate xdrs as _xdrs;
            impl #impl_gen #ident #type_gen #where_gen {
                pub fn discriminant(&self) -> _xdrs::reexport::Option<#discriminant_ty> {
                    match self {
                        #(#variant_to_discriminant),*
                    }
                }
            }

            impl #impl_gen _xdrs::Encode for #ident #type_gen #where_gen {
                fn encode<__W: _xdrs::reexport::Write>(&self, out: &mut __W) -> _xdrs::reexport::Result<usize, _xdrs::Error> {
                    match self {
                        #(#variant_to_encode),*
                    }
                }
            }
        };
    };
    Ok(out)
}

fn check_repr_i32(input: &SpannedValue<EncodeReceiver>) -> syn::Result<()> {
    let repr_attr = input
        .attrs
        .iter()
        .find(|a| a.meta.path().is_ident("repr"))
        .ok_or_else(|| {
            syn::Error::new(input.span(), "`repr(i32)` attribute not specified")
        })?;

    let mut repr_i32 = false;
    repr_attr.parse_nested_meta(|meta| {
        // #[repr(i32)]
        if meta.path.is_ident("i32") {
            repr_i32 = true;
            return Ok(());
        }

        Ok(())
    })?;

    if !repr_i32 {
        return Err(syn::Error::new(
            input.span(),
            "`repr(i32)` attribute is required",
        ));
    }

    Ok(())
}
