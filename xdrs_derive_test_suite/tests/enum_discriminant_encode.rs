mod utils;

use std::io::Write;

use xdrs::{Encode, Error};

#[test]
fn common() {
    #[derive(Encode)]
    #[repr(i32)]
    enum Derived {
        VariantA,
        VariantB = 1,
        VariantC = 255,
    }

    #[repr(i32)]
    enum Manual {
        VariantA,
        VariantB = 1,
        VariantC = 255,
    }

    impl Encode for Manual {
        fn encode<W: Write>(&self, out: &mut W) -> Result<usize, Error> {
            match self {
                Manual::VariantA => 0.encode(out),
                Manual::VariantB => 1.encode(out),
                Manual::VariantC => 255.encode(out),
            }
        }
    }

    let act = Derived::VariantA;
    let exp = Manual::VariantA;
    utils::assert_pack_eq(act, exp);

    let act = Derived::VariantB;
    let exp = Manual::VariantB;
    utils::assert_pack_eq(act, exp);

    let act = Derived::VariantC;
    let exp = Manual::VariantC;
    utils::assert_pack_eq(act, exp);

    assert_eq!(Derived::VariantA.as_i32(), 0);
    assert_eq!(Derived::VariantB.as_i32(), 1);
    assert_eq!(Derived::VariantC.as_i32(), 255);
}
