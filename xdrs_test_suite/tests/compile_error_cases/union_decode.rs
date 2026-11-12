use xdrs::Decode;

#[derive(Decode)]
union IntOrFloat {
    i: u32,
    f: f32,
}

fn main() {}
