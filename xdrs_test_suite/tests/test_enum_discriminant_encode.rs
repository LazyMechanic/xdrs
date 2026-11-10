use xdrs::{Encode, Encoder, test_utils::assert_encode_eq};

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
        fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
            match self {
                Manual::VariantA => 0.encode(encoder),
                Manual::VariantB => 1.encode(encoder),
                Manual::VariantC => 255.encode(encoder),
            }
        }
    }

    let act = Derived::VariantA;
    let exp = Manual::VariantA;
    assert_encode_eq(act, exp);

    let act = Derived::VariantB;
    let exp = Manual::VariantB;
    assert_encode_eq(act, exp);

    let act = Derived::VariantC;
    let exp = Manual::VariantC;
    assert_encode_eq(act, exp);
}
