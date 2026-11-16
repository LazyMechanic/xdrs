use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// union-type-spec:
///    "union" union-body
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<UnionTypeSpec> {
    todo!()
}

pub struct UnionTypeSpec {}
