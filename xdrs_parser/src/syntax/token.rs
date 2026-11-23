use logos::Span;
use winnow::{
    ModalResult, Parser,
    error::{ContextError, ErrMode},
    stream::Stream,
    token::any,
};

use crate::token::Token as LexerToken;

macro_rules! define_tokens {
    ($(($lexer_token:expr, $raw_token:literal) pub struct $type_name:ident;)*) => {
        $(
            #[doc = concat!('`', $raw_token, '`')]
            #[derive(Debug, Clone, Eq, PartialEq)]
            pub struct $type_name {
                span: Span,
            }

            impl $type_name {
                pub fn span(&self) -> &Span {
                    &self.span
                }

                pub fn parse<'a, 't>(input: &mut TokenStream<'a, 't>) -> ModalResult<Self> {
                    let expected = $lexer_token;
                    let err_cx = expected.error_context();
                    let parser = move |input: &mut TokenStream<'a, 't>| {
                        let checkpoint = input.checkpoint();
                        let next = any.parse_next(input)?;
                        if *next.token() == expected {
                            Ok(Self {
                                span: next.span().clone(),
                            })
                        } else {
                            input.reset(&checkpoint);
                            Err(ErrMode::Backtrack(ContextError::new()))
                        }
                    };

                    parser.context(err_cx).parse_next(input)
                }
            }

            #[allow(non_snake_case)]
            pub fn $type_name (input: &mut TokenStream<'_, '_>) -> ModalResult<$type_name> {
                $type_name::parse(input)
            }
        )*
    };
}

define_tokens! {
    (LexerToken::Const, "const")            pub struct ConstToken;
    (LexerToken::Struct, "struct")          pub struct StructToken;
    (LexerToken::Enum, "enum")              pub struct EnumToken;
    (LexerToken::Union, "union")            pub struct UnionToken;
    (LexerToken::Switch, "switch")          pub struct SwitchToken;
    (LexerToken::Case, "case")              pub struct CaseToken;
    (LexerToken::Default, "default")        pub struct DefaultToken;
    (LexerToken::Typedef, "typedef")        pub struct TypedefToken;
    (LexerToken::Void, "void")              pub struct VoidToken;
    (LexerToken::Bool, "bool")              pub struct BoolToken;
    (LexerToken::Int, "int")                pub struct IntToken;
    (LexerToken::Hyper, "hyper")            pub struct HyperToken;
    (LexerToken::Unsigned, "unsigned")      pub struct UnsignedToken;
    (LexerToken::Float, "float")            pub struct FloatToken;
    (LexerToken::Double, "double")          pub struct DoubleToken;
    (LexerToken::Quadruple, "quadruple")    pub struct QuadrupleToken;
    (LexerToken::Opaque, "opaque")          pub struct OpaqueToken;
    (LexerToken::String, "string")          pub struct StringToken;
    (LexerToken::LParen, "(")               pub struct LParenToken;
    (LexerToken::RParen, ")")               pub struct RParenToken;
    (LexerToken::LBrace, "{")               pub struct LBraceToken;
    (LexerToken::RBrace, "}")               pub struct RBraceToken;
    (LexerToken::LBracket, "[")             pub struct LBracketToken;
    (LexerToken::RBracket, "]")             pub struct RBracketToken;
    (LexerToken::LAngle, "<")               pub struct LAngleToken;
    (LexerToken::RAngle, ">")               pub struct RAngleToken;
    (LexerToken::Semi, ";")                 pub struct SemiToken;
    (LexerToken::Colon, ":")                pub struct ColonToken;
    (LexerToken::Eq, "=")                   pub struct EqToken;
    (LexerToken::Star, "*")                 pub struct StarToken;
    (LexerToken::Comma, ",")                pub struct CommaToken;
}

#[allow(non_snake_case)]
macro_rules! Token {
    [const] => {$crate::syntax::token::ConstToken};
    [struct] => {$crate::syntax::token::StructToken};
    [enum] => {$crate::syntax::token::EnumToken};
    [union] => {$crate::syntax::token::UnionToken};
    [switch] => {$crate::syntax::token::SwitchToken};
    [case] => {$crate::syntax::token::CaseToken};
    [default] => {$crate::syntax::token::DefaultToken};
    [typedef] => {$crate::syntax::token::TypedefToken};
    [void] => {$crate::syntax::token::VoidToken};
    [bool] => {$crate::syntax::token::BoolToken};
    [int] => {$crate::syntax::token::IntToken};
    [hyper] => {$crate::syntax::token::HyperToken};
    [unsigned] => {$crate::syntax::token::UnsignedToken};
    [float] => {$crate::syntax::token::FloatToken};
    [double] => {$crate::syntax::token::DoubleToken};
    [quadruple] => {$crate::syntax::token::QuadrupleToken};
    [opaque] => {$crate::syntax::token::OpaqueToken};
    [string] => {$crate::syntax::token::StringToken};
    ['('] => {$crate::syntax::token::LParenToken};
    [')'] => {$crate::syntax::token::RParenToken};
    ['{'] => {$crate::syntax::token::LBraceToken};
    ['}'] => {$crate::syntax::token::RBraceToken};
    ['['] => {$crate::syntax::token::LBracketToken};
    [']'] => {$crate::syntax::token::RBracketToken};
    [<] => {$crate::syntax::token::LAngleToken};
    [>] => {$crate::syntax::token::RAngleToken};
    [;] => {$crate::syntax::token::SemiToken};
    [:] => {$crate::syntax::token::ColonToken};
    [=] => {$crate::syntax::token::EqToken};
    [*] => {$crate::syntax::token::StarToken};
    [,] => {$crate::syntax::token::CommaToken};
}
pub(crate) use Token;

use crate::token::TokenStream;
