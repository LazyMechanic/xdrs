use winnow::{
    ModalResult, Parser,
    combinator::{alt, cut_err, empty, opt},
    seq,
};

use crate::{
    syntax::{ident::Ident, token::Token, type_spec::TypeSpec, value::Value},
    token::TokenStream,
};

/// ```text
/// declaration:
///      type-specifier identifier
///    | type-specifier identifier "[" value "]"
///    | type-specifier identifier "<" [ value ] ">"
///    | "opaque" identifier "[" value "]"
///    | "opaque" identifier "<" [ value ] ">"
///    | "string" identifier "<" [ value ] ">"
///    | type-specifier "*" identifier
///    | "void"
/// ```
#[allow(non_snake_case)]
pub fn Declaration<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<Declaration<'t>> {
    alt((
        DeclarationOpaqueFix.map(Declaration::OpaqueFix),
        DeclarationOpaqueVar.map(Declaration::OpaqueVar),
        DeclarationString.map(Declaration::String),
        DeclarationVoid.map(Declaration::Void),
        DeclarationTypedVar.map(Box::new).map(Declaration::TypedVar),
        DeclarationTypedFix.map(Box::new).map(Declaration::TypedFix),
        DeclarationTyped.map(Box::new).map(Declaration::Typed),
    ))
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn DeclarationTyped<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<DeclarationTyped<'t>> {
    seq! {
        DeclarationTyped {
            ty: TypeSpec,
            star: opt(Token![*]),
            ident: cut_err(Ident),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn DeclarationTypedFix<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<DeclarationTypedFix<'t>> {
    seq! {
        DeclarationTypedFix {
            ty: TypeSpec,
            ident: Ident,
            lbracket: Token!['['],
            len: cut_err(Value),
            rbracket: cut_err(Token![']']),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn DeclarationTypedVar<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<DeclarationTypedVar<'t>> {
    seq! {
        DeclarationTypedVar {
            ty: TypeSpec,
            ident: Ident,
            langle: Token![<],
            len: opt(Value),
            rangle: cut_err(Token![>]),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn DeclarationOpaqueFix<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<DeclarationOpaqueFix<'t>> {
    seq! {
        DeclarationOpaqueFix {
            ty: Token![opaque],
            ident: Ident,
            lbracket: Token!['['],
            len: cut_err(Value),
            rbracket: cut_err(Token![']']),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn DeclarationOpaqueVar<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<DeclarationOpaqueVar<'t>> {
    seq! {
        DeclarationOpaqueVar {
            ty: Token![opaque],
            ident: Ident,
            langle: Token![<],
            len: opt(Value),
            rangle: cut_err(Token![>]),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn DeclarationString<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<DeclarationString<'t>> {
    seq! {
        DeclarationString {
            ty: Token![string],
            ident: Ident,
            langle: Token![<],
            len: opt(Value),
            rangle: cut_err(Token![>]),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn DeclarationVoid(
    input: &mut TokenStream<'_, '_>,
) -> ModalResult<DeclarationVoid> {
    seq! {
        DeclarationVoid {
            ty: Token![void],
        }
    }
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Declaration<'a> {
    Typed(Box<DeclarationTyped<'a>>),
    TypedFix(Box<DeclarationTypedFix<'a>>),
    TypedVar(Box<DeclarationTypedVar<'a>>),
    OpaqueFix(DeclarationOpaqueFix<'a>),
    OpaqueVar(DeclarationOpaqueVar<'a>),
    String(DeclarationString<'a>),
    Void(DeclarationVoid),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeclarationTyped<'t> {
    pub ty: TypeSpec<'t>,
    pub star: Option<Token![*]>,
    pub ident: Ident<'t>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeclarationTypedFix<'t> {
    pub ty: TypeSpec<'t>,
    pub ident: Ident<'t>,
    pub lbracket: Token!['['],
    pub len: Value<'t>,
    pub rbracket: Token![']'],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeclarationTypedVar<'t> {
    pub ty: TypeSpec<'t>,
    pub ident: Ident<'t>,
    pub langle: Token![<],
    pub len: Option<Value<'t>>,
    pub rangle: Token![>],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeclarationOpaqueFix<'t> {
    pub ty: Token![opaque],
    pub ident: Ident<'t>,
    pub lbracket: Token!['['],
    pub len: Value<'t>,
    pub rbracket: Token![']'],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeclarationOpaqueVar<'t> {
    pub ty: Token![opaque],
    pub ident: Ident<'t>,
    pub langle: Token![<],
    pub len: Option<Value<'t>>,
    pub rangle: Token![>],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeclarationString<'a> {
    pub ty: Token![string],
    pub ident: Ident<'a>,
    pub langle: Token![<],
    pub len: Option<Value<'a>>,
    pub rangle: Token![>],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeclarationVoid {
    pub ty: Token![void],
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::utils::{asserts, define_asserts, parse_str};

    define_asserts! {
        type DeclarationVoid;
        pub fn declaration_void_asserts(acc) {
            asserts! {
                acc;
                type DeclarationVoid;
                "void": DeclarationVoid { .. },
            }
        }
    }

    define_asserts! {
        type DeclarationString;
        pub fn declaration_string_asserts(acc) {
            asserts! {
                acc;
                type DeclarationString;
                "string ident <>":
                    DeclarationString { ident, len: None, .. }
                    if &**ident == "ident",
                "string ident <123>":
                    DeclarationString { ident, len: Some(Value::Const(len)), .. }
                    if &**ident == "ident" && len.value() == 123,
                "string ident <CONST>":
                    DeclarationString { ident, len: Some(Value::Ident(len)), .. }
                    if &**ident == "ident" && &**len == "CONST",
            }
        }
    }

    define_asserts! {
        type DeclarationOpaqueVar<'a>;
        pub fn declaration_opaque_var_asserts(acc) {
            asserts! {
                acc;
                type DeclarationOpaqueVar<'a>;
                "opaque ident <>":
                    DeclarationOpaqueVar { ident, len: None, .. }
                    if &**ident == "ident",
                "opaque ident <123>":
                    DeclarationOpaqueVar { ident, len: Some(Value::Const(len)), .. }
                    if &**ident == "ident" && len.value() == 123,
                "opaque ident <CONST>":
                    DeclarationOpaqueVar { ident, len: Some(Value::Ident(len)), .. }
                    if &**ident == "ident" && &**len == "CONST",
            }
        }
    }

    define_asserts! {
        type DeclarationOpaqueFix<'a>;
        pub fn declaration_opaque_fix_asserts(acc) {
            asserts! {
                acc;
                type DeclarationOpaqueFix<'a>;
                "opaque ident [123]":
                    DeclarationOpaqueFix { ident, len: Value::Const(len), .. }
                    if &**ident == "ident" && len.value() == 123,
                "opaque ident [CONST]":
                    DeclarationOpaqueFix { ident, len: Value::Ident(len), .. }
                    if &**ident == "ident" && &**len == "CONST",
            }
        }
    }

    define_asserts! {
        type DeclarationTypedVar<'a>;
        pub fn declaration_typed_var_asserts(acc) {
            asserts! {
                acc;
                type DeclarationTypedVar<'a>;
                "int ident <>":
                    DeclarationTypedVar { ty: TypeSpec::Int(_), ident, len: None, .. }
                    if &**ident == "ident",
                "int ident <123>":
                    DeclarationTypedVar { ty: TypeSpec::Int(_), ident, len: Some(Value::Const(len)), .. }
                    if &**ident == "ident" && len.value() == 123,
                "int ident <CONST>":
                    DeclarationTypedVar { ty: TypeSpec::Int(_), ident, len: Some(Value::Ident(len)), .. }
                    if &**ident == "ident" && &**len == "CONST",
            }
        }
    }

    define_asserts! {
        type DeclarationTypedFix<'a>;
        pub fn declaration_typed_fix_asserts(acc) {
            asserts! {
                acc;
                type DeclarationTypedFix<'a>;
                "int ident [123]":
                    DeclarationTypedFix { ty: TypeSpec::Int(_), ident, len: Value::Const(len), .. }
                    if &**ident == "ident" && len.value() == 123,
                "int ident [CONST]":
                    DeclarationTypedFix { ty: TypeSpec::Int(_), ident, len: Value::Ident(len), .. }
                    if &**ident == "ident" && &**len == "CONST",
            }
        }
    }

    define_asserts! {
        type DeclarationTyped<'a>;
        pub fn declaration_typed_asserts(acc) {
            asserts! {
                acc;
                type DeclarationTyped<'a>;
                "int ident":
                    DeclarationTyped { ty: TypeSpec::Int(_), ident, .. }
                    if &**ident == "ident",
            }
        }
    }

    define_asserts! {
        type Declaration<'a>;
        pub fn declaration_asserts(acc) {
            for (input, assert) in declaration_typed_asserts() {
                asserts! { acc; type Declaration; { input.clone() }: Declaration::Typed(v) => assert(&v) }
            }
            for (input, assert) in declaration_typed_fix_asserts() {
                asserts! { acc; type Declaration; { input.clone() }: Declaration::TypedFix(v) => assert(&v) }
            }
            for (input, assert) in declaration_typed_var_asserts() {
                asserts! { acc; type Declaration; { input.clone() }: Declaration::TypedVar(v) => assert(&v) }
            }
            for (input, assert) in declaration_opaque_fix_asserts() {
                asserts! { acc; type Declaration; { input.clone() }: Declaration::OpaqueFix(v) => assert(&v) }
            }
            for (input, assert) in declaration_opaque_var_asserts() {
                asserts! { acc; type Declaration; { input.clone() }: Declaration::OpaqueVar(v) => assert(&v) }
            }
            for (input, assert) in declaration_string_asserts() {
                asserts! { acc; type Declaration; { input.clone() }: Declaration::String(v) => assert(&v) }
            }
            for (input, assert) in declaration_void_asserts() {
                asserts! { acc; type Declaration; { input.clone() }: Declaration::Void(v) => assert(&v) }
            }
        }
    }

    #[test]
    fn declaration() {
        for (input, assert) in declaration_asserts() {
            assert(&parse_str(Declaration, input).unwrap());
        }
    }

    #[test]
    fn declaration_typed() {
        for (input, assert) in declaration_typed_asserts() {
            assert(&parse_str(DeclarationTyped, input).unwrap());
        }

        let _ = parse_str(DeclarationTyped, "int").unwrap_err();
        let _ = parse_str(DeclarationTyped, "ident").unwrap_err();
        let _ = parse_str(DeclarationTyped, "int ident <>").unwrap_err();
        let _ = parse_str(DeclarationTyped, "int ident <123>").unwrap_err();
        let _ = parse_str(DeclarationTyped, "int ident [123]").unwrap_err();
    }

    #[test]
    fn declaration_typed_var() {
        for (input, assert) in declaration_typed_var_asserts() {
            assert(&parse_str(DeclarationTypedVar, input).unwrap());
        }

        let _ = parse_str(DeclarationTypedVar, "ident <123>").unwrap_err();
        let _ = parse_str(DeclarationTypedVar, "int <123>").unwrap_err();
        let _ = parse_str(DeclarationTypedVar, "int ident 123>").unwrap_err();
        let _ = parse_str(DeclarationTypedVar, "int ident >").unwrap_err();
        let _ = parse_str(DeclarationTypedVar, "int ident <123").unwrap_err();
        let _ = parse_str(DeclarationTypedVar, "int ident <").unwrap_err();
    }

    #[test]
    fn declaration_typed_fix() {
        for (input, assert) in declaration_typed_fix_asserts() {
            assert(&parse_str(DeclarationTypedFix, input).unwrap());
        }

        let _ = parse_str(DeclarationTypedFix, "ident [123]").unwrap_err();
        let _ = parse_str(DeclarationTypedFix, "int [123]").unwrap_err();
        let _ = parse_str(DeclarationTypedFix, "int ident 123]").unwrap_err();
        let _ = parse_str(DeclarationTypedFix, "int ident []").unwrap_err();
        let _ = parse_str(DeclarationTypedFix, "int ident [123").unwrap_err();
    }

    #[test]
    fn declaration_opaque_fix() {
        for (input, assert) in declaration_opaque_fix_asserts() {
            assert(&parse_str(DeclarationOpaqueFix, input).unwrap());
        }

        let _ = parse_str(DeclarationOpaqueFix, "ident [123]").unwrap_err();
        let _ = parse_str(DeclarationOpaqueFix, "opaque [123]").unwrap_err();
        let _ =
            parse_str(DeclarationOpaqueFix, "opaque ident 123]").unwrap_err();
        let _ = parse_str(DeclarationOpaqueFix, "opaque ident []").unwrap_err();
        let _ =
            parse_str(DeclarationOpaqueFix, "opaque ident [123").unwrap_err();
    }

    #[test]
    fn declaration_opaque_var() {
        for (input, assert) in declaration_opaque_var_asserts() {
            assert(&parse_str(DeclarationOpaqueVar, input).unwrap());
        }

        let _ = parse_str(DeclarationOpaqueVar, "ident <123>").unwrap_err();
        let _ = parse_str(DeclarationOpaqueVar, "opaque <123>").unwrap_err();
        let _ =
            parse_str(DeclarationOpaqueVar, "opaque ident 123>").unwrap_err();
        let _ = parse_str(DeclarationOpaqueVar, "opaque ident >").unwrap_err();
        let _ =
            parse_str(DeclarationOpaqueVar, "opaque ident <123").unwrap_err();
        let _ = parse_str(DeclarationOpaqueVar, "opaque ident <").unwrap_err();
    }

    #[test]
    fn declaration_string() {
        for (input, assert) in declaration_string_asserts() {
            assert(&parse_str(DeclarationString, input).unwrap());
        }

        let _ = parse_str(DeclarationString, "ident <123>").unwrap_err();
        let _ = parse_str(DeclarationString, "string <123>").unwrap_err();
        let _ = parse_str(DeclarationString, "string ident 123>").unwrap_err();
        let _ = parse_str(DeclarationString, "string ident >").unwrap_err();
        let _ = parse_str(DeclarationString, "string ident <123").unwrap_err();
        let _ = parse_str(DeclarationString, "string ident <").unwrap_err();
    }

    #[test]
    fn declaration_void() {
        for (input, assert) in declaration_void_asserts() {
            assert(&parse_str(DeclarationVoid, input).unwrap());
        }

        let _ = parse_str(DeclarationVoid, "qwe").unwrap_err();
    }
}
