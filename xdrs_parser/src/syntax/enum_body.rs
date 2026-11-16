use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// enum-body:
///    "{"
///       ( identifier "=" value )
///       ( "," identifier "=" value )*
///    "}"
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<EnumBody> {
    todo!()
}

pub struct EnumBody {}
