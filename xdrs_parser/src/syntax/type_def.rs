use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// type-def:
///      "typedef" declaration ";"
///    | "enum" identifier enum-body ";"
///    | "struct" identifier struct-body ";"
///    | "union" identifier union-body ";"
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<TypeDef> {
    todo!()
}

pub struct TypeDef {}
