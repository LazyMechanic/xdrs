use xdrs::Discriminant;

#[test]
fn as_discriminant() {
    #[derive(Discriminant)]
    #[repr(i32)]
    enum Enum {
        VariantA,
        VariantB = 5,
        VariantC,
        VariantD = i32::MAX,
        VariantE = i32::MIN,
        VariantF = 42,
    }

    assert_eq!(Enum::VariantA.as_discriminant(), 0);
    assert_eq!(Enum::VariantB.as_discriminant(), 5);
    assert_eq!(Enum::VariantC.as_discriminant(), 6);
    assert_eq!(Enum::VariantD.as_discriminant(), i32::MAX);
    assert_eq!(Enum::VariantE.as_discriminant(), i32::MIN);
    assert_eq!(Enum::VariantF.as_discriminant(), 42);
}

#[test]
fn from_discriminant() {
    #[derive(Debug, PartialEq, Discriminant)]
    #[repr(i32)]
    enum Enum {
        VariantA,
        VariantB = 5,
        VariantC,
        VariantD = i32::MAX,
        VariantE = i32::MIN,
        VariantF = 42,
    }

    assert_eq!(Enum::from_discriminant(0), Some(Enum::VariantA));
    assert_eq!(Enum::from_discriminant(5), Some(Enum::VariantB));
    assert_eq!(Enum::from_discriminant(6), Some(Enum::VariantC));
    assert_eq!(Enum::from_discriminant(i32::MAX), Some(Enum::VariantD));
    assert_eq!(Enum::from_discriminant(i32::MIN), Some(Enum::VariantE));
    assert_eq!(Enum::from_discriminant(42), Some(Enum::VariantF));
    assert_eq!(Enum::from_discriminant(2156186), None);
}
