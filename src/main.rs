use std::{env, fs, time};

use huffman::{encode_bytes};

mod huffman;
mod bitstream;

fn main() {
    let args:Vec<String> = env::args().collect();
    let filepath = &args[1];
    //dbg!(args);

    let contents = fs::read(filepath).expect("File not found");

    let start_time = time::Instant::now();
    let bitstream = encode_bytes(&contents);
    let elapsed_time = start_time.elapsed().as_millis();
    println!("Bytes unencoded: [{}] Bytes encoded:[{}] Compression ratio:[{}]\nTime:[{}]ms Speed:[{}]MB/s",contents.len(), bitstream.bytes_written(), (bitstream.bytes_written() as f32) / (contents.len() as f32), elapsed_time, ((contents.len() as f32) / 1000f32) / (elapsed_time as f32));

    //bitstream.flush();
    //println!("{}",bitstream);
}
