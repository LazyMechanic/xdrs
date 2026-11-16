use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// enum-type-spec:
///    "enum" enum-body
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<EnumTypeSpec> {
    todo!()
}

pub struct EnumTypeSpec {}
