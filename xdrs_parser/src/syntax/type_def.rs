use winnow::{
    ModalResult, Parser,
    combinator::{alt, cut_err},
    seq,
};

use crate::{
    syntax::{
        declaration::Declaration, enum_body::EnumBody, ident::Ident,
        struct_body::StructBody, token::Token, union_body::UnionBody,
    },
    token::TokenStream,
};

/// ```text
/// type-def:
///      "typedef" declaration ";"
///    | "enum" identifier enum-body ";"
///    | "struct" identifier struct-body ";"
///    | "union" identifier union-body ";"
/// ```
#[allow(non_snake_case)]
pub fn TypeDef<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<TypeDef<'t>> {
    alt((
        TypeDefAlias.map(TypeDef::Alias),
        TypeDefEnum.map(TypeDef::Enum),
        TypeDefStruct.map(TypeDef::Struct),
        TypeDefUnion.map(TypeDef::Union),
    ))
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn TypeDefAlias<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<TypeDefAlias<'t>> {
    seq! {
        TypeDefAlias {
            typedef: Token![typedef],
            declaration: cut_err(Declaration),
            semi: cut_err(Token![;]),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn TypeDefEnum<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<TypeDefEnum<'t>> {
    seq! {
        TypeDefEnum {
            enum_: Token![enum],
            ident: Ident,
            body: cut_err(EnumBody),
            semi: cut_err(Token![;]),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn TypeDefStruct<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<TypeDefStruct<'t>> {
    seq! {
        TypeDefStruct {
            struct_: Token![struct],
            ident: Ident,
            body: cut_err(StructBody),
            semi: cut_err(Token![;]),
        }
    }
    .parse_next(input)
}

#[allow(non_snake_case)]
pub fn TypeDefUnion<'a, 't>(
    input: &mut TokenStream<'a, 't>,
) -> ModalResult<TypeDefUnion<'t>> {
    seq! {
        TypeDefUnion {
            union_: Token![union],
            ident: Ident,
            body: cut_err(UnionBody),
            semi: cut_err(Token![;]),
        }
    }
    .parse_next(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TypeDef<'t> {
    Alias(TypeDefAlias<'t>),
    Enum(TypeDefEnum<'t>),
    Struct(TypeDefStruct<'t>),
    Union(TypeDefUnion<'t>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeDefAlias<'t> {
    pub typedef: Token![typedef],
    pub declaration: Declaration<'t>,
    pub semi: Token![;],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeDefEnum<'t> {
    pub enum_: Token![enum],
    pub ident: Ident<'t>,
    pub body: EnumBody<'t>,
    pub semi: Token![;],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeDefStruct<'t> {
    pub struct_: Token![struct],
    pub ident: Ident<'t>,
    pub body: StructBody<'t>,
    pub semi: Token![;],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeDefUnion<'t> {
    pub union_: Token![union],
    pub ident: Ident<'t>,
    pub body: UnionBody<'t>,
    pub semi: Token![;],
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::syntax::{
        declaration::tests::declaration_asserts,
        enum_body::tests::enum_body_asserts,
        struct_body::tests::struct_body_asserts,
        union_body::tests::union_body_asserts,
        utils::{asserts, define_asserts, parse_str},
    };

    define_asserts! {
        type TypeDefAlias<'a>;
        pub fn type_def_alias_asserts(acc) {
            for (decl_input, decl_assert) in declaration_asserts() {
                asserts! {
                    acc;
                    type TypeDefAlias<'a>;
                    { format!("typedef {decl_input};") }:
                        TypeDefAlias { declaration, .. }
                        => decl_assert(declaration),
                }
            }
        }
    }

    define_asserts! {
        type TypeDefEnum<'a>;
        pub fn type_def_enum_asserts(acc) {
            for (body_input, body_assert) in enum_body_asserts() {
                asserts! {
                    acc;
                    type TypeDefEnum<'a>;
                    { format!("enum ident {body_input};") }:
                        TypeDefEnum { ident, body, .. }
                        if &**ident == "ident"
                        => body_assert(body),
                }
            }
        }
    }

    define_asserts! {
        type TypeDefStruct<'a>;
        pub fn type_def_struct_asserts(acc) {
            for (body_input, body_assert) in struct_body_asserts() {
                asserts! {
                    acc;
                    type TypeDefStruct<'a>;
                    { format!("struct ident {body_input};") }:
                        TypeDefStruct { ident, body, .. }
                        if &**ident == "ident"
                        => body_assert(body),
                }
            }
        }
    }

    define_asserts! {
        type TypeDefUnion<'a>;
        pub fn type_def_union_asserts(acc) {
            for (body_input, body_assert) in union_body_asserts() {
                asserts! {
                    acc;
                    type TypeDefUnion<'a>;
                    { format!("union ident {body_input};") }:
                        TypeDefUnion { ident, body, .. }
                        if &**ident == "ident"
                        => body_assert(body),
                }
            }
        }
    }

    define_asserts! {
        type TypeDef<'a>;
        pub fn type_def_asserts(acc) {
            for (input, assert) in type_def_alias_asserts() {
                asserts! { acc; type TypeDef<'a>; { input.clone() }: TypeDef::Alias(v) => assert(v) }
            }

            for (input, assert) in type_def_enum_asserts() {
                asserts! { acc; type TypeDef<'a>; { input.clone() }: TypeDef::Enum(v) => assert(v) }
            }

            for (input, assert) in type_def_struct_asserts() {
                asserts! { acc; type TypeDef<'a>; { input.clone() }: TypeDef::Struct(v) => assert(v) }
            }

            for (input, assert) in type_def_union_asserts() {
                asserts! { acc; type TypeDef<'a>; { input.clone() }: TypeDef::Union(v) => assert(v) }
            }
        }
    }

    #[test]
    fn type_def_alias() {
        for (input, assert) in type_def_alias_asserts() {
            assert(&parse_str(TypeDefAlias, input).unwrap());
        }

        let _ = parse_str(TypeDefAlias, "typedef alias ident;").unwrap();

        let _ = parse_str(TypeDefAlias, "alias ident;").unwrap_err();
        let _ = parse_str(TypeDefAlias, "typedef ;").unwrap_err();
        let _ = parse_str(TypeDefAlias, "typedef alias ident").unwrap_err();
    }

    #[test]
    fn type_def_enum() {
        for (input, assert) in type_def_enum_asserts() {
            assert(&parse_str(TypeDefEnum, input).unwrap());
        }

        let _ = parse_str(
            TypeDefEnum,
            r#"
            enum ident {
              VARIANT_A = 1,
              VARIANT_B = 128
            };
            "#,
        )
        .unwrap();

        let _ = parse_str(TypeDefEnum, "ident { VARIANT = 1 };").unwrap_err();
        let _ = parse_str(TypeDefEnum, "enum ident ;").unwrap_err();
        let _ =
            parse_str(TypeDefEnum, "enum ident { VARIANT = 1 }").unwrap_err();
    }

    #[test]
    fn type_def_struct() {
        for (input, assert) in type_def_struct_asserts() {
            assert(&parse_str(TypeDefStruct, input).unwrap());
        }

        let _ = parse_str(
            TypeDefStruct,
            r#"
            struct ident {
                int a;
                float b;
            };
            "#,
        )
        .unwrap();

        let _ = parse_str(TypeDefStruct, "ident { int field; };").unwrap_err();
        let _ = parse_str(TypeDefStruct, "struct ident ;").unwrap_err();
        let _ = parse_str(TypeDefStruct, "struct ident { int field; }")
            .unwrap_err();
    }

    #[test]
    fn type_def_union() {
        for (input, assert) in type_def_union_asserts() {
            assert(&parse_str(TypeDefUnion, input).unwrap());
        }

        let _ = parse_str(
            TypeDefUnion,
            r#"
            union ident switch (int v) {
                case 1: int a;
                case 2: float b;
                default: void;
            };
            "#,
        )
        .unwrap();

        let _ = parse_str(
            TypeDefUnion,
            r#"
            ident switch (int v) {
                case 1: int a;
                case 2: float b;
                default: void;
            };
            "#,
        )
        .unwrap_err();
        let _ = parse_str(
            TypeDefUnion,
            r#"
            union switch (int v) {
                case 1: int a;
                case 2: float b;
                default: void;
            };
            "#,
        )
        .unwrap_err();
        let _ = parse_str(
            TypeDefUnion,
            r#"
            union ident (int v) {
                case 1: int a;
                case 2: float b;
                default: void;
            };
            "#,
        )
        .unwrap_err();
        let _ = parse_str(
            TypeDefUnion,
            r#"
            union ident switch int v) {
                case 1: int a;
                case 2: float b;
                default: void;
            };
            "#,
        )
        .unwrap_err();
        let _ = parse_str(
            TypeDefUnion,
            r#"
            union ident switch () {
                case 1: int a;
                case 2: float b;
                default: void;
            };
            "#,
        )
        .unwrap_err();
        let _ = parse_str(
            TypeDefUnion,
            r#"
            union ident switch (int v {
                case 1: int a;
                case 2: float b;
                default: void;
            };
            "#,
        )
        .unwrap_err();
        let _ = parse_str(
            TypeDefUnion,
            r#"
            union ident switch (int v);
            "#,
        )
        .unwrap_err();
        let _ = parse_str(
            TypeDefUnion,
            r#"
            union ident switch (int v) {
                case 1: int a;
                case 2: float b;
                default: void;
            }
            "#,
        )
        .unwrap_err();
    }

    #[test]
    fn type_def() {
        for (input, assert) in type_def_asserts() {
            assert(&parse_str(TypeDef, input).unwrap());
        }
    }
}
