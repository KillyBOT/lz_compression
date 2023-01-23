mod huffman;
mod bitstream;
mod lzw;
mod lz;
mod lz77;

use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    two: String,
    #[arg(long)]
    one: String,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}

fn main() {
    
    //use std::{env, fs, time};
    //use crate::huffman::{encode_bytes_huffman, decode_bytes_huffman};


    // let contents = fs::read("enwik8").expect("File could not be opened and/or read");

    // let start_time = time::Instant::now();
    // let encoded_bytes = encode_bytes_huffman(&contents, 1<<18, 11);
    // let elapsed_time = start_time.elapsed().as_millis();

    // println!("Bytes unencoded: [{}] Bytes encoded:[{}] Compression ratio:[{}]\nTime:[{}]ms Speed:[{}]MB/s",contents.len(), encoded_bytes.len(), (encoded_bytes.len() as f32) / (contents.len() as f32), elapsed_time, ((contents.len() as f32) / 1000f32) / (elapsed_time as f32));
    
    // let start_time = time::Instant::now();
    // let decoded_bytes = decode_bytes_huffman(&encoded_bytes);
    // let elapsed_time = start_time.elapsed().as_millis();

    // println!("Decompression time:[{}]ms Speed:[{}]MB/s", elapsed_time, ((encoded_bytes.len() as f32) / 1000f32) / (elapsed_time as f32));
    
    // assert!(contents.len() == decoded_bytes.len(), "Number of bytes different after encoding and decoding");
    // for i in 0..contents.len(){
    //     assert!(contents[i] == decoded_bytes[i], "Bytes different after encoding and decoding");
    // }
}
