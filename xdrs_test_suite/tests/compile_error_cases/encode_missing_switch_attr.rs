use xdrs::Encode;

#[derive(Encode)]
#[repr(i32)]
enum Discriminant {
    Int,
    String,
    Void,
}

#[derive(Encode)]
enum Derived {
    #[xdrs(discriminant = "Discriminant::Int")]
    Int(i32),
    #[xdrs(discriminant = "Discriminant::String")]
    String(String),
    #[xdrs(discriminant = "Discriminant::Void")]
    Void,
    #[xdrs(default)]
    Default,
}

fn main() {}
