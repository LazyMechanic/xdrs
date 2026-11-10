use xdrs::{Decode, test_utils::assert_decode};

#[test]
fn common() {
    #[derive(Debug, PartialEq, Decode)]
    #[repr(i32)]
    enum Derived {
        VariantA,
        VariantB = 1,
        VariantC = 255,
    }

    let exp = Derived::VariantA;
    assert_decode(&[0x00, 0x00, 0x00, 0x00], exp);

    let exp = Derived::VariantB;
    assert_decode(&[0x00, 0x00, 0x00, 0x01], exp);

    let exp = Derived::VariantC;
    assert_decode(&[0x00, 0x00, 0x00, 0xff], exp);
}
