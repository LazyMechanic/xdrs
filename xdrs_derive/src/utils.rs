use darling::util::SpannedValue;
use syn::{Generics, spanned::Spanned};

use crate::receivers::{
    DeriveReceiver, DiscriminantVariantReceiver, FieldReceiver, VariantReceiver,
};

pub fn check_repr_i32(input: &SpannedValue<DeriveReceiver>) -> syn::Result<()> {
    let repr_attr = input
        .attrs
        .iter()
        .find(|a| a.meta.path().is_ident("repr"))
        .ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "`#[repr(i32)]` attribute not specified",
            )
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

pub fn check_no_generics(generics: &Generics) -> syn::Result<()> {
    if generics.type_params().any(|_| true)
        || generics.lifetimes().any(|_| true)
        || generics.const_params().any(|_| true)
    {
        return Err(syn::Error::new(
            generics.span(),
            "generics are not supported",
        ));
    }

    Ok(())
}

pub fn check_struct_is_struct(
    input: &SpannedValue<DeriveReceiver>,
    data: &darling::ast::Fields<SpannedValue<FieldReceiver>>,
) -> syn::Result<()> {
    if !data.style.is_struct() {
        return Err(syn::Error::new(
            input.span(),
            "only structs with named fields are supported",
        ));
    }

    Ok(())
}

pub fn check_struct_attrs(
    input: &SpannedValue<DeriveReceiver>,
    _data: &darling::ast::Fields<SpannedValue<FieldReceiver>>,
) -> syn::Result<()> {
    if let Some(attr) = &input.switch_attr {
        return Err(syn::Error::new(
            attr.span(),
            "unsupported attribute in struct",
        ));
    }

    Ok(())
}

pub fn check_enum_union_attrs(
    input: &SpannedValue<DeriveReceiver>,
    data: &[SpannedValue<VariantReceiver>],
) -> syn::Result<()> {
    if input.switch_attr.is_none() {
        return Err(syn::Error::new(
            input.span(),
            "missing required `#[xdrs(switch = ...)]` attribute for union",
        ));
    }

    let mut has_default = false;
    for d in data {
        match (&d.discriminant_attr, &d.default_attr) {
            // #[xdrs(discriminant = "")]
            (Some(_), def_attr) if !def_attr.is_present() => {
                if !((d.fields.is_tuple() && d.fields.fields.len() == 1)
                    || d.fields.is_unit())
                {
                    return Err(syn::Error::new(
                        d.span(),
                        "only variants with 1 unnamed field or without fields (unit) are supported",
                    ));
                }
            }

            // #[xdrs(default)]
            (None, def_attr) if def_attr.is_present() => {
                if !((d.fields.is_tuple() && d.fields.fields.len() == 1)
                    || d.fields.is_unit())
                {
                    return Err(syn::Error::new(
                        d.span(),
                        "only variants with 1 unnamed field or without fields (unit) are supported",
                    ));
                }

                if has_default {
                    return Err(syn::Error::new(
                        d.default_attr.span(),
                        "multiple `#[xdrs(default)]` attributes is not allowed",
                    ));
                }

                has_default = true;
            }

            // Both present
            (Some(_), def_attr) if def_attr.is_present() => {
                return Err(syn::Error::new(
                    d.span(),
                    "only one attribute either `#[xdrs(discriminant)]` or `#[xdrs(default)]` allowed",
                ));
            }

            // Missing both
            (_, _) => {
                return Err(syn::Error::new(
                    d.span(),
                    "missing required `#[xdrs(discriminant)]` or `#[xdrs(default)]` attribute",
                ));
            }
        }
    }

    Ok(())
}

pub fn check_enum_discriminant_attrs(
    input: &SpannedValue<DeriveReceiver>,
    _data: &[SpannedValue<DiscriminantVariantReceiver>],
) -> syn::Result<()> {
    if input.switch_attr.is_some() {
        return Err(syn::Error::new(
            input.span(),
            "unsupported attribute in enum with discriminants",
        ));
    }

    Ok(())
}

pub fn check_enum_union_has_variants(
    input: &SpannedValue<DeriveReceiver>,
    data: &[SpannedValue<VariantReceiver>],
) -> syn::Result<()> {
    // If no variants
    if data.is_empty() {
        return Err(syn::Error::new(
            input.span(),
            "enum with value must contain at least one variant with `#[xdrs(discriminant = \"...\")]` attribute",
        ));
    }

    // If there are only one variant but without `#[xdrs(discriminant = "...")]` attribute
    if data.len() == 1
        && let Some(d) = data.first()
        && d.discriminant_attr.is_none()
    {
        return Err(syn::Error::new(
            input.span(),
            "enum with value must contain at least one variant with `#[xdrs(discriminant = \"...\")]` attribute",
        ));
    }

    Ok(())
}

pub fn check_enum_discriminant_has_variants(
    input: &SpannedValue<DeriveReceiver>,
    data: &[SpannedValue<DiscriminantVariantReceiver>],
) -> syn::Result<()> {
    if data.is_empty() {
        return Err(syn::Error::new(
            input.span(),
            "enum with discriminants must contain at least one variant",
        ));
    }

    Ok(())
}
