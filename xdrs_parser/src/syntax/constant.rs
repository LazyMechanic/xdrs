use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// constant:
///    decimal-constant | hexadecimal-constant | octal-constant
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<Const> {
    todo!()
}

pub struct Const {}
