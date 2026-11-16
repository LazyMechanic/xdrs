use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// declaration:
///      type-specifier identifier
///    | type-specifier identifier "[" value "]"
///    | type-specifier identifier "<" [ value ] ">"
///    | "opaque" identifier "[" value "]"
///    | "opaque" identifier "<" [ value ] ">"
///    | "string" identifier "<" [ value ] ">"
///    | type-specifier "*" identifier
///    | "void"
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<Declaration> {
    todo!()
}

pub struct Declaration {}
