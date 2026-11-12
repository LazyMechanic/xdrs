use xdrs::Decode;

#[derive(Decode)]
enum Discriminant {
    Int,
    String,
    Void,
}

fn main() {}
