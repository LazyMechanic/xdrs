use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// type-specifier:
///      [ "unsigned" ] "int"
///    | [ "unsigned" ] "hyper"
///    | "float"
///    | "double"
///    | "quadruple"
///    | "bool"
///    | enum-type-spec
///    | struct-type-spec
///    | union-type-spec
///    | identifier
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<TypeSpec> {
    todo!()
}

pub struct TypeSpec {}
