use std::env;
use std::fs;

use huffman::*;

pub mod huffman;
fn main() {
    let args:Vec<String> = env::args().collect();
    let filepath = &args[1];
    //dbg!(args);

    let contents = fs::read(filepath).expect("File not found");

    let freq_table = build_frequency_table(&contents);
    let huffman_tree = build_huffman_tree(freq_table);
    println!("{:?}",huffman_tree);
}
