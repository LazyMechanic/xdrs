use std::ops::Deref;

use darling::FromMeta;

/// `#[xdrs(switch = "path::to::Enum")]`
#[derive(Debug, Clone)]
pub struct XdrsSwitchAttr(syn::Path);

impl FromMeta for XdrsSwitchAttr {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self(syn::Path::from_string(value)?))
    }
}

impl Deref for XdrsSwitchAttr {
    type Target = syn::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// `#[xdrs(discriminant = "VariantA")]`
#[derive(Debug, Clone)]
pub struct XdrsDiscriminantAttr(syn::Expr);

impl FromMeta for XdrsDiscriminantAttr {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self(syn::Expr::from_string(value)?))
    }
}

impl Deref for XdrsDiscriminantAttr {
    type Target = syn::Expr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
