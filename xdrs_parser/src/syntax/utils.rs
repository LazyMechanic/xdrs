use std::fmt::Debug;

use winnow::{
    Parser,
    combinator::trace,
    error::{ContextError, ErrMode},
};

pub use self::punctuated::Punctuated;
#[cfg(test)]
pub(crate) use self::tests::{asserts, define_asserts, parse_str};
use crate::token::TokenStream;

pub fn separated0<'a, 't: 'a, Item, Punct, ItemParser, PunctParser>(
    item_parser: ItemParser,
    punct_parser: PunctParser,
) -> impl Parser<TokenStream<'a, 't>, Punctuated<Item, Punct>, ErrMode<ContextError>>
where
    ItemParser: Parser<TokenStream<'a, 't>, Item, ErrMode<ContextError>>,
    PunctParser: Parser<TokenStream<'a, 't>, Punct, ErrMode<ContextError>>,
{
    trace(
        "separated0",
        punctuated::punctuated0(item_parser, punct_parser, false),
    )
}

pub fn separated1<'a, 't: 'a, Item, Punct, ItemParser, PunctParser>(
    item_parser: ItemParser,
    punct_parser: PunctParser,
) -> impl Parser<TokenStream<'a, 't>, Punctuated<Item, Punct>, ErrMode<ContextError>>
where
    ItemParser: Parser<TokenStream<'a, 't>, Item, ErrMode<ContextError>>,
    PunctParser: Parser<TokenStream<'a, 't>, Punct, ErrMode<ContextError>>,
{
    trace(
        "separated1",
        punctuated::punctuated1(item_parser, punct_parser, false),
    )
}

pub fn terminated0<'a, 't: 'a, Item, Punct, ItemParser, PunctParser>(
    item_parser: ItemParser,
    punct_parser: PunctParser,
) -> impl Parser<TokenStream<'a, 't>, Punctuated<Item, Punct>, ErrMode<ContextError>>
where
    ItemParser: Parser<TokenStream<'a, 't>, Item, ErrMode<ContextError>>,
    PunctParser: Parser<TokenStream<'a, 't>, Punct, ErrMode<ContextError>>,
{
    trace(
        "terminated0",
        punctuated::punctuated0(item_parser, punct_parser, true),
    )
}

pub fn terminated1<'a, 't: 'a, Item, Punct, ItemParser, PunctParser>(
    item_parser: ItemParser,
    punct_parser: PunctParser,
) -> impl Parser<TokenStream<'a, 't>, Punctuated<Item, Punct>, ErrMode<ContextError>>
where
    ItemParser: Parser<TokenStream<'a, 't>, Item, ErrMode<ContextError>>,
    PunctParser: Parser<TokenStream<'a, 't>, Punct, ErrMode<ContextError>>,
{
    trace(
        "terminated1",
        punctuated::punctuated1(item_parser, punct_parser, true),
    )
}

mod punctuated {
    use winnow::{
        Parser,
        combinator::{cut_err, opt},
        error::{ContextError, ErrMode, ParserError},
        stream::Stream,
    };

    use crate::token::TokenStream;

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Punctuated<T, P> {
        inner: Vec<(T, P)>,
        last: Option<Box<T>>,
    }

    impl<T, P> Punctuated<T, P> {
        pub fn iter(&self) -> Iter<'_, T, P> {
            Iter {
                inner: self.inner.iter(),
                last: self.last.as_ref().map(Box::as_ref).into_iter(),
            }
        }

        pub fn into_iter(self) -> IntoIter<T, P> {
            IntoIter {
                inner: self.inner.into_iter(),
                last: self.last.map(|v| *v).into_iter(),
            }
        }

        fn empty_or_trailing(&self) -> bool {
            self.last.is_none()
        }

        fn push_item(&mut self, item: T) {
            assert!(
                self.empty_or_trailing(),
                "`Punctuated::push_item`: cannot push item if `Punctuated` is missing trailing punctuation",
            );

            self.last = Some(Box::new(item));
        }

        fn push_punct(&mut self, punctuation: P) {
            let last = self.last.take().expect("`Punctuated::push_punct`: cannot push separation if `Punctuated` is empty or already has trailing punctuation");
            self.inner.push((*last, punctuation));
        }
    }

    impl<'a, T, P> IntoIterator for &'a Punctuated<T, P> {
        type Item = &'a T;
        type IntoIter = Iter<'a, T, P>;

        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    }

    impl<T, P> IntoIterator for Punctuated<T, P> {
        type Item = T;
        type IntoIter = IntoIter<T, P>;

        fn into_iter(self) -> Self::IntoIter {
            self.into_iter()
        }
    }

    pub struct Iter<'a, T, P> {
        inner: std::slice::Iter<'a, (T, P)>,
        last: std::option::IntoIter<&'a T>,
    }

    impl<'a, T, P> Clone for Iter<'a, T, P> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
                last: self.last.clone(),
            }
        }
    }

    impl<'a, T, P> Iterator for Iter<'a, T, P> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner
                .next()
                .map(|pair| &pair.0)
                .or_else(|| self.last.next())
        }
    }

    pub struct IntoIter<T, P> {
        inner: std::vec::IntoIter<(T, P)>,
        last: std::option::IntoIter<T>,
    }

    impl<T: Clone, P: Clone> Clone for IntoIter<T, P> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
                last: self.last.clone(),
            }
        }
    }

    impl<T, P> Iterator for IntoIter<T, P> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner
                .next()
                .map(|pair| pair.0)
                .or_else(|| self.last.next())
        }
    }

    pub(super) fn punctuated0<
        'a,
        't: 'a,
        Item,
        Punct,
        ItemParser,
        PunctParser,
    >(
        mut item_parser: ItemParser,
        mut punct_parser: PunctParser,
        trailing_required: bool,
    ) -> impl Parser<
        TokenStream<'a, 't>,
        Punctuated<Item, Punct>,
        ErrMode<ContextError>,
    >
    where
        ItemParser: Parser<TokenStream<'a, 't>, Item, ErrMode<ContextError>>,
        PunctParser: Parser<TokenStream<'a, 't>, Punct, ErrMode<ContextError>>,
    {
        move |input: &mut TokenStream<'a, 't>| {
            let mut acc = Punctuated {
                inner: vec![],
                last: None,
            };

            punctuated_in(
                item_parser.by_ref(),
                punct_parser.by_ref(),
                trailing_required,
                &mut acc,
            )
            .parse_next(input)?;

            Ok(acc)
        }
    }

    pub(super) fn punctuated1<
        'a,
        't: 'a,
        Item,
        Punct,
        ItemParser,
        PunctParser,
    >(
        mut item_parser: ItemParser,
        mut punct_parser: PunctParser,
        trailing_required: bool,
    ) -> impl Parser<
        TokenStream<'a, 't>,
        Punctuated<Item, Punct>,
        ErrMode<ContextError>,
    >
    where
        ItemParser: Parser<TokenStream<'a, 't>, Item, ErrMode<ContextError>>,
        PunctParser: Parser<TokenStream<'a, 't>, Punct, ErrMode<ContextError>>,
    {
        move |input: &mut TokenStream<'a, 't>| {
            let mut acc = Punctuated {
                inner: vec![],
                last: None,
            };

            // First element
            {
                let (item, punct) = cut_err(punctuated_step(
                    item_parser.by_ref(),
                    punct_parser.by_ref(),
                    trailing_required,
                ))
                .parse_next(input)?;

                acc.push_item(item);
                if let Some(punct) = punct {
                    acc.push_punct(punct);
                }
            }

            punctuated_in(
                item_parser.by_ref(),
                punct_parser.by_ref(),
                trailing_required,
                &mut acc,
            )
            .parse_next(input)?;

            Ok(acc)
        }
    }

    fn punctuated_step<'a, 't: 'a, Item, Punct, ItemParser, PunctParser>(
        mut item_parser: ItemParser,
        mut punct_parser: PunctParser,
        trailing_required: bool,
    ) -> impl Parser<TokenStream<'a, 't>, (Item, Option<Punct>), ErrMode<ContextError>>
    where
        ItemParser: Parser<TokenStream<'a, 't>, Item, ErrMode<ContextError>>,
        PunctParser: Parser<TokenStream<'a, 't>, Punct, ErrMode<ContextError>>,
    {
        move |input: &mut TokenStream<'a, 't>| {
            let item = item_parser.parse_next(input)?;

            if trailing_required {
                let punct = cut_err(punct_parser.by_ref()).parse_next(input)?;
                Ok((item, Some(punct)))
            } else {
                let punct = opt(punct_parser.by_ref()).parse_next(input)?;
                Ok((item, punct))
            }
        }
    }

    fn punctuated_in<'a, 't: 'a, Item, Punct, ItemParser, PunctParser>(
        mut item_parser: ItemParser,
        mut punct_parser: PunctParser,
        trailing_required: bool,
        acc: &mut Punctuated<Item, Punct>,
    ) -> impl Parser<TokenStream<'a, 't>, (), ErrMode<ContextError>>
    where
        ItemParser: Parser<TokenStream<'a, 't>, Item, ErrMode<ContextError>>,
        PunctParser: Parser<TokenStream<'a, 't>, Punct, ErrMode<ContextError>>,
    {
        move |input: &mut TokenStream<'a, 't>| {
            loop {
                let checkpoint = input.checkpoint();
                match punctuated_step(
                    item_parser.by_ref(),
                    punct_parser.by_ref(),
                    trailing_required,
                )
                .parse_next(input)
                {
                    Ok((item, punct)) => {
                        acc.push_item(item);
                        if let Some(punct) = punct {
                            acc.push_punct(punct);
                        }
                    }
                    Err(err)
                        if ParserError::<TokenStream<'a, 't>>::is_backtrack(
                            &err,
                        ) =>
                    {
                        input.reset(&checkpoint);
                        return Ok(());
                    }
                    Err(err) => return Err(err),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use miette::ErrReport;
    use winnow::{
        Parser,
        error::{ContextError, ErrMode},
    };

    use crate::{
        error::SyntaxError,
        token::{TokenStream, tokenize},
    };

    pub fn parse_str<'t, P, O>(mut parser: P, src: &'t str) -> miette::Result<O>
    where
        P: for<'a> Parser<TokenStream<'a, 't>, O, ErrMode<ContextError>>,
    {
        let tokens = tokenize(src)?;
        parser.parse(&mut tokens.as_slice()).map_err(|err| {
            let token_idx = err.offset();

            let byte_span: Range<usize> =
                if let Some(tok) = tokens.get(token_idx) {
                    tok.span().clone()
                } else if let Some(last_tok) = tokens.last() {
                    last_tok.span().end..last_tok.span().end
                } else {
                    // No input
                    0..0
                };

            let span = byte_span.into();

            let mut message = err.inner().to_string();
            if message.is_empty() {
                message = "syntax error".into();
            }

            ErrReport::new(SyntaxError { span, message })
                .with_source_code(src.to_owned())
        })
    }

    /// Syntax:
    /// ```
    /// # stringify! {
    /// asserts! {
    ///     acc;
    ///     type NonTerminal<'a>;
    ///     "123": NonTerminal::Int(_),
    ///     "qwe": NonTerminal::Str(s) if s == "qwe",
    ///     "const": NonTerminal::Const(c) => assert!(c.literal() == "const"),
    /// }
    /// # }
    /// ```
    macro_rules! asserts {
        // Start TT muncher
        ($acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; $($tt:tt)*) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; $($tt)*)
        };

        // End
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?;) => {};

        // { <expr> }: <pat1> [| <pat2>]* if <condition> => <arm>,
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; { $name:expr }: $($pat:pat_param)|+ if $condition:expr => $arm:expr, $($rest:tt)*) => {
            {
                fn __hktb<F>(f: F) -> Box<dyn for<'__f, $($($lt),+)?> Fn(&'__f $ty$(<$($lt),+>)?) + Send + Sync + 'static>
                where
                    F: for<'__f, $($($lt),+)?> Fn(&'__f $ty$(<$($lt),+>)?) + Send + Sync + 'static,
                {
                    Box::new(f)
                }

                $acc.insert(
                    std::borrow::Cow::from($name),
                    __hktb(|val| {
                        assert_matches::assert_matches!(val, $($pat)|+ if $condition => $arm);
                    })
                );
            }
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; $($rest)*);
        };

        // { <expr> }: <pat1> [| <pat2>]* if <condition> => <arm>
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; { $name:expr } : $($pat:pat_param)|+ if $condition:expr => $arm:expr) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; { $name} : $($pat)|+ if $condition => $arm,);
        };


        // { <expr> }: <pat1> [| <pat2>]* => <arm>,
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; { $name:expr } : $($pat:pat_param)|+ => $arm:expr, $($rest:tt)*) => {
            {
                fn __hktb<F>(f: F) -> Box<dyn for<'__f, $($($lt),+)?> Fn(&'__f $ty$(<$($lt),+>)?) + Send + Sync + 'static>
                where
                    F: for<'__f, $($($lt),+)?> Fn(&'__f $ty$(<$($lt),+>)?) + Send + Sync + 'static,
                {
                    Box::new(f)
                }

                $acc.insert(
                    std::borrow::Cow::from($name),
                    __hktb(|val| {
                        assert_matches::assert_matches!(val, $($pat)|+ => $arm);
                    })
                );
            }
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; $($rest)*);
        };

        // { <expr> }: <pat1> [| <pat2>]* => <arm>
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; { $name:expr } : $($pat:pat_param)|+ => $arm:expr) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; { $name} : $($pat)|+ => $arm,);
        };

        // { <expr> }: <pat1> [| <pat2>]* if <condition>,
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; { $name:expr } : $($pat:pat_param)|+ if $condition:expr, $($rest:tt)*) => {
            {
                fn __hktb<F>(f: F) -> Box<dyn for<'__f, $($($lt),+)?> Fn(&'__f $ty$(<$($lt),+>)?) + Send + Sync + 'static>
                where
                    F: for<'__f, $($($lt),+)?> Fn(&'__f $ty$(<$($lt),+>)?) + Send + Sync + 'static,
                {
                    Box::new(f)
                }
                $acc.insert(
                    std::borrow::Cow::from($name),
                    __hktb(|val| {
                        assert_matches::assert_matches!(val, $($pat)|+ if $condition);
                    })
                );
            }
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; $($rest)*);
        };

        // { <expr> }: <pat1> [| <pat2>]* if <condition>
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; { $name:expr } : $($pat:pat_param)|+ if $condition:expr) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; $($pat)|+ if $condition,);
        };

        // { <expr> }: <pat1> [| <pat2>]*,
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; { $name:expr } : $($pat:pat_param)|+, $($rest:tt)*) => {
            {
                fn __hktb<F>(f: F) -> Box<dyn for<'__f, $($($lt),+)?> Fn(&'__f $ty$(<$($lt),+>)?) + Send + Sync + 'static>
                where
                    F: for<'__f, $($($lt),+)?> Fn(&'__f $ty$(<$($lt),+>)?) + Send + Sync + 'static,
                {
                    Box::new(f)
                }
                $acc.insert(
                    std::borrow::Cow::from($name),
                    __hktb(|val| {
                        assert_matches::assert_matches!(val, $($pat)|+);
                    })
                );
            }
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; $($rest)*);
        };

        // { <expr> }: <pat1> [| <pat2>]*
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; { $name:expr } : $($pat:pat_param)|+) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; $($pat)|+,);
        };

        ////////////////

        // <literal>: <pat1> [| <pat2>]* if <condition> => <arm>,
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; $name:literal : $($pat:pat_param)|+ if $condition:expr => $arm:expr, $($rest:tt)*) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; { $name } : $($pat)|+ if $condition => $arm, $($rest)*);
        };

        // <literal>: <pat1> [| <pat2>]* if <condition> => <arm>
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; $name:literal : $($pat:pat_param)|+ if $condition:expr => $arm:expr) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; { $name } : $($pat)|+ if $condition => $arm);
        };

        // <literal>: <pat1> [| <pat2>]* => <arm>,
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; $name:literal : $($pat:pat_param)|+ => $arm:expr, $($rest:tt)*) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; { $name } : $($pat)|+ => $arm, $($rest)*);
        };

        // <literal>: <pat1> [| <pat2>]* => <arm>
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; $name:literal : $($pat:pat_param)|+ => $arm:expr) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; { $name } : $($pat)|+ => $arm);
        };

        // <literal>: <pat1> [| <pat2>]* if <condition>,
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; $name:literal : $($pat:pat_param)|+ if $condition:expr, $($rest:tt)*) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; { $name } : $($pat)|+ if $condition, $($rest)*);
        };

        // <literal>: <pat1> [| <pat2>]* if <condition>
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; $name:literal : $($pat:pat_param)|+ if $condition:expr) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; { $name } : $($pat)|+ if $condition);
        };

        // <literal>: <pat1> [| <pat2>]*,
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; $name:literal : $($pat:pat_param)|+, $($rest:tt)*) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; { $name } : $($pat)|+, $($rest)*);
        };

        // <literal>: <pat1> [| <pat2>]*
        (@parse $acc:expr; type $ty:ident$(<$($lt:lifetime),+>)?; $name:literal : $($pat:pat_param)|+) => {
            asserts!(@parse $acc; type $ty$(<$($lt),+>)?; { $name } : $($pat)|+);
        };
    }
    pub(crate) use asserts;

    /// Syntax:
    /// ```
    /// # stringify! {
    /// define_asserts! {
    ///     type NonTerminal<'a>;
    ///     pub fn non_terminal_asserts(acc) {
    ///         asserts! {
    ///             acc;
    ///             type NonTerminal<'a>,
    ///             // ...
    ///         }
    ///     }
    /// }
    ///
    /// #[test]
    /// fn test {
    ///     for (input, assert) in non_terminal_asserts() {
    ///         assert(&parse_str(NonTerminal, input).unwrap());
    ///     }
    /// }
    /// # }
    /// ```
    macro_rules! define_asserts {
        (
            type $testable_ident:ident $(<$($testable_lt:lifetime),+>)?;
            pub fn $fn_ident:ident($acc_ident:ident) { $($body:tt)* }
        ) => {
            pub fn $fn_ident() -> &'static std::collections::HashMap<
                std::borrow::Cow<'static, str>,
                Box<dyn for <'__f, $($($testable_lt),+)?> Fn(&'__f $testable_ident $(<$($testable_lt),+>)?) + Send + Sync + 'static>,
            > {
                static ASSERTS: std::sync::LazyLock<
                    std::collections::HashMap<
                        std::borrow::Cow<'static, str>,
                        Box<dyn for <'__f, $($($testable_lt),+)?> Fn(&'__f $testable_ident $(<$($testable_lt),+>)?) + Send + Sync + 'static>
                    >,
                > = std::sync::LazyLock::new(|| {
                    let mut $acc_ident = std::collections::HashMap::new();
                    $($body)*
                    $acc_ident
                });
                &ASSERTS
            }
        };
    }
    pub(crate) use define_asserts;
}
