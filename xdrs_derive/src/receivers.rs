use darling::{FromDeriveInput, FromField, FromVariant, util::SpannedValue};
use proc_macro2::Span;

use crate::attrs::{XdrsDiscriminantAttr, XdrsSwitchAttr};

#[derive(Clone, FromDeriveInput)]
#[darling(attributes(xdrs), forward_attrs(repr))]
pub struct DeriveReceiver {
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub data: darling::ast::Data<
        SpannedValue<VariantReceiver>,
        SpannedValue<FieldReceiver>,
    >,
    pub attrs: Vec<syn::Attribute>,

    // #[xdrs(switch = "path::to::Enum")] over enum union-like
    #[darling(default, rename = "switch")]
    pub switch_attr: Option<SpannedValue<XdrsSwitchAttr>>,
}

/// Struct fields
/// ```text
/// a: i32,
/// b: String,
/// ...
/// ```
#[derive(Clone, FromField)]
#[darling(attributes(xdrs))]
pub struct FieldReceiver {
    pub ident: Option<syn::Ident>,
}

/// Enum variants
/// ```text
/// VariantA(StructA),
/// VariantB = 123,
/// ...
/// ```
#[derive(Debug, Clone, FromVariant)]
#[darling(attributes(xdrs))]
pub struct VariantReceiver {
    pub ident: syn::Ident,
    pub fields: darling::ast::Fields<SpannedValue<VariantFieldReceiver>>,
    pub discriminant: Option<syn::Expr>,

    // #[xdrs(default)]
    #[darling(default, rename = "default")]
    pub default_attr: darling::util::Flag,
    // #[xdrs(discriminant = "VariantA")]
    #[darling(default, rename = "discriminant")]
    pub discriminant_attr: Option<SpannedValue<XdrsDiscriminantAttr>>,
}

#[derive(Debug, Clone, FromField)]
pub struct VariantFieldReceiver {
    pub ty: syn::Type,
}

#[derive(Clone)]
pub struct DiscriminantVariantReceiver {
    pub ident: syn::Ident,

    span: Span,
}

impl DiscriminantVariantReceiver {
    pub fn into_spanned(self) -> SpannedValue<Self> {
        let span = self.span;
        SpannedValue::new(self, span)
    }
}

impl TryFrom<SpannedValue<VariantReceiver>> for DiscriminantVariantReceiver {
    type Error = syn::Error;

    fn try_from(
        value: SpannedValue<VariantReceiver>,
    ) -> Result<Self, Self::Error> {
        let span = value.span();
        let value = value.into_inner();

        if value.discriminant_attr.is_some() {
            return Err(syn::Error::new(
                span,
                "to use a `#[xdrs(discriminant = \"...\")]` attribute on variants you must \
                specify the discriminant type with `#[xdrs(switch)]` attribute on the enum",
            ));
        }

        if value.discriminant_attr.is_some() {
            return Err(syn::Error::new(
                span,
                "to use a `#[xdrs(default)` attribute on variants you must \
                specify the discriminant type with `#[xdrs(switch)]` attribute on the enum",
            ));
        }

        if !value.fields.is_unit() {
            return Err(syn::Error::new(
                span,
                "non-unit variants require explicit `#[xdrs(discriminant = \"...\")]` attribute",
            ));
        }

        Ok(Self {
            ident: value.ident,
            span,
        })
    }
}
