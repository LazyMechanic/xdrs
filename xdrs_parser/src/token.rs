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
#[logos(skip(r"[ \t\r\n\f]+"))] // skip whitespaces
pub enum Token<'a> {
    #[token("const")]
    Const,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("union")]
    Union,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("typedef")]
    Typedef,

    #[token("void")]
    Void,
    #[token("bool")]
    Bool,
    #[token("int")]
    Int,
    #[token("hyper")]
    Hyper,
    #[token("unsigned")]
    Unsigned,
    #[token("float")]
    Float,
    #[token("double")]
    Double,
    #[token("quadruple")]
    Quadruple,
    #[token("opaque")]
    Opaque,
    #[token("string")]
    String,

    /// `{` Token
    #[token("{")]
    LBrace,
    /// `]` Token
    #[token("}")]
    RBrace,
    /// `[` Token
    #[token("[")]
    LBracket,
    /// `]` Token
    #[token("]")]
    RBracket,
    /// `<` Token
    #[token("<")]
    LAngle,
    /// `>` Token
    #[token(">")]
    RAngle,
    /// `;` Token
    #[token(";")]
    Semi,
    /// `:` Token
    #[token(":")]
    Colon,
    /// `=` Token
    #[token("=")]
    Eq,

    /// One line comments `// ...`
    #[regex(r"//[^\n]*", allow_greedy = true)]
    LineComment(&'a str),

    /// Multiline comments `/* ... */`
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/")]
    BlockComment(&'a str),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),

    /// `0xFF` / `077` / `123`
    #[regex(r"0x[0-9a-fA-F]+|0[0-7]*|[1-9][0-9]*")]
    IntLit(&'a str),
}

pub struct SpannedToken<'a> {
    token: Token<'a>,
    span: Span,
}
