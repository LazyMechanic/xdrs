use xdrs::Encode;

pub fn assert_pack_eq<A: Encode, E: Encode>(actual: A, expected: E) {
    let mut act_buf = Vec::new();
    let mut exp_buf = Vec::new();

    assert_eq!(
        actual.encode(&mut act_buf).unwrap(),
        expected.encode(&mut exp_buf).unwrap()
    );
    assert_eq!(act_buf, exp_buf);
}
