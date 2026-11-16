use crate::token::TokenizeError;

#[non_exhaustive]
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("failed to parse schema")]
pub enum ParseError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Tokenize(#[from] TokenizeError),
}
