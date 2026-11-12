use xdrs::Discriminant;

#[derive(Discriminant)]
enum Enum {
    Int,
    String,
    Void,
}

fn main() {}
