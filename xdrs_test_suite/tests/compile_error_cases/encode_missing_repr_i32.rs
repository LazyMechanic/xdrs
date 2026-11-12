use xdrs::Encode;

#[derive(Encode)]
enum Discriminant {
    Int,
    String,
    Void,
}

fn main() {}
