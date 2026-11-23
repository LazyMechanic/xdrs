use std::{
    fmt::{Display, Formatter},
    ops::Deref,
};

use logos::Span;
use winnow::{
    ModalResult, Parser,
    error::{ContextError, ErrMode},
    stream::Stream,
    token::any,
};

use crate::token::{Token, TokenStream};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ident<'a> {
    name: &'a str,
    span: Span,
}

impl Deref for Ident<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.name
    }
}

impl Display for Ident<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

#[allow(non_snake_case)]
pub fn Ident<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<Ident<'t>, ContextError> {
    let checkpoint = input.checkpoint();
    let next = any.parse_next(input)?;
    if let Token::Ident(name) = next.token() {
        Ok(Ident {
            name,
            span: next.span().clone(),
        })
    } else {
        input.reset(&checkpoint);
        Err(ErrMode::Backtrack(ContextError::new()))
    }
}

#[cfg(test)]
pub mod tests {
    pub use super::*;
    use crate::syntax::utils::{asserts, define_asserts, parse_str};

    define_asserts! {
        type Ident<'a>;
        pub fn ident_asserts(acc) {
            asserts! {
                acc;
                type Ident<'a>;
                "qwerty123": Ident { name: "qwerty123", .. },
                "QWERTY123": Ident { name: "QWERTY123", .. },
                "_qWeRtY123": Ident { name: "_qWeRtY123", .. },
                "__": Ident { name: "__", .. },
            }
        }
    }

    #[test]
    fn ident() {
        for (input, assert) in ident_asserts() {
            assert(&parse_str(Ident, input).unwrap());
        }

        let _ = parse_str(Ident, "-qwerty").unwrap_err();
        let _ = parse_str(Ident, "012345").unwrap_err();
        let _ = parse_str(Ident, "*").unwrap_err();
        let _ = parse_str(Ident, ":").unwrap_err();
    }
}
