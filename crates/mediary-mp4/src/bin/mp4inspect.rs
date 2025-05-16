use std::fs::File;

use mediary_mp4::Mp4;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let mp4 = Mp4::read(&mut File::open(&path).unwrap()).unwrap();

    println!("{mp4:#?}");
}
