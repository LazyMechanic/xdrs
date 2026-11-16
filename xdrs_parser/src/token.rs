use logos::{Logos, Span};
use miette::SourceSpan;

pub(crate) fn tokenize(
    input: &str,
) -> Result<Vec<SpannedToken<'_>>, TokenizeError> {
    let mut tokens = Vec::new();

    for (token_res, span) in Token::lexer(input).spanned() {
        tokens.push(SpannedToken {
            token: token_res?,
            span,
        });
    }

    Ok(tokens)
}

#[non_exhaustive]
#[derive(
    Default, Debug, Clone, PartialEq, thiserror::Error, miette::Diagnostic,
)]
#[error("failed to tokenize input")]
pub enum TokenizeError {
    #[error("processed invalid token")]
    InvalidToken {
        #[label("invalid syntax")]
        span: SourceSpan,
    },
    #[error("unknown error")]
    #[default]
    UnknownError,
}

impl TokenizeError {
    fn invalid_token<'a>(lex: &mut logos::Lexer<'a, Token<'a>>) -> Self {
        Self::InvalidToken {
            span: SourceSpan::from(lex.span()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Logos)]
#[logos(error(TokenizeError, callback = TokenizeError::invalid_token))]
pub enum Token<'a> {
    __(&'a ()),
}

pub struct SpannedToken<'a> {
    token: Token<'a>,
    span: Span,
}
