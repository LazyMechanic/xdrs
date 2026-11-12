use xdrs::Encode;

#[derive(Encode)]
#[xdrs(switch = "i32")]
enum Enum {}

fn main() {}
