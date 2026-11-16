use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// struct-body:
///    "{"
///       ( declaration ";" )
///       ( declaration ";" )*
///    "}"
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<StructBody> {
    todo!()
}

pub struct StructBody {}
