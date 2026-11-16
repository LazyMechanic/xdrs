use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// struct-type-spec:
///    "struct" struct-body
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<StructTypeSpec> {
    todo!()
}

pub struct StructTypeSpec {}
