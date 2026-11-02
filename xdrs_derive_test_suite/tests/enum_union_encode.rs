mod utils;

use std::io::Write;

use xdrs::{Encode, Error, Void};

#[test]
fn common() {
    #[derive(Debug, Eq, PartialEq, Encode)]
    #[repr(i32)]
    enum Discriminant {
        Int,
        String,
        Void,
    }

    #[derive(Encode)]
    #[xdrs(switch = "Discriminant")]
    enum Derived {
        #[xdrs(discriminant = "Int")]
        Int(i32),
        #[xdrs(discriminant = "String")]
        String(String),
        #[xdrs(discriminant = "Void")]
        Void,
        #[xdrs(default)]
        Default,
    }

    enum Manual {
        Int(i32),
        String(String),
        Void,
        Default,
    }

    impl Encode for Manual {
        fn encode<W: Write>(&self, out: &mut W) -> Result<usize, Error> {
            match self {
                Manual::Int(v) => v.encode(out),
                Manual::String(v) => v.encode(out),
                Manual::Void => Void.encode(out),
                Manual::Default => Void.encode(out),
            }
        }
    }

    let act = Derived::Int(123);
    let exp = Manual::Int(123);
    assert_eq!(act.discriminant(), Some(Discriminant::Int));
    utils::assert_pack_eq(act, exp);

    let act = Derived::String("hello world".into());
    let exp = Manual::String("hello world".into());
    assert_eq!(act.discriminant(), Some(Discriminant::String));
    utils::assert_pack_eq(act, exp);

    let act = Derived::Void;
    let exp = Manual::Void;
    assert_eq!(act.discriminant(), Some(Discriminant::Void));
    utils::assert_pack_eq(act, exp);

    let act = Derived::Default;
    let exp = Manual::Default;
    assert_eq!(act.discriminant(), None);
    utils::assert_pack_eq(act, exp);
}
