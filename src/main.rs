use std::env;
use std::fs;

use huffman::*;

pub mod huffman;
fn main() {
    let args:Vec<String> = env::args().collect();
    let filepath = &args[1];
    //dbg!(args);

    let contents = fs::read(filepath).expect("File not found");

    let bitstream = encode_bytes(&contents);
    println!("Bytes unencoded: [{}] Bytes encoded:[{}] Compression ratio:[{}]",contents.len(), bitstream.bytes_written(), (bitstream.bytes_written() as f32) / (contents.len() as f32));

    //bitstream.flush();
    //println!("{}",bitstream);
}
