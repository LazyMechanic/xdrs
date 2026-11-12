use xdrs::Decode;

#[derive(Decode)]
#[xdrs(switch = "i32")]
enum Enum {
    #[xdrs(default)]
    Default,
}

fn main() {}
