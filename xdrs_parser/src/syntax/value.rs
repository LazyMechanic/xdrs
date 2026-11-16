use winnow::ModalResult;

use crate::token::SpannedToken;

/// ```text
/// value:
///      constant
///    | identifier
/// ```
pub fn parse(input: &[SpannedToken<'_>]) -> ModalResult<Value> {
    todo!()
}

pub struct Value {}
