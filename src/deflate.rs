use crate::huffman::{HuffmanEncoder, HuffmanDecoder, HuffmanSymbol};
use crate::lz77::{lz77_compress_simple, lz77_decompress};

fn extra_bits_for_length_symbol(symbol: HuffmanSymbol) -> usize {
    match symbol {
        257..=264 => 0,
        265..=268 => 1,
        269..=272 => 2,
        273..=276 => 3,
        277..=280 => 4,
        281..=284 => 5,
        _ => 0
    }
}

fn extra_bits_for_dist_symbol(symbol: HuffmanSymbol) -> usize {
    match symbol {
        0..=3 => 0,
        4 | 5 => 1,
        6 | 7 => 2,
        8 | 9 => 3,
        10 | 11 => 4,
        12 | 13 => 5,
        14 | 15 => 6,
        16 | 17 => 7,
        18 | 19 => 8,
        20 | 21 => 9,
        22 | 23 => 10,
        24 | 25 => 11,
        26 | 27 => 12,
        28 | 29 => 13,
        _ => 0
    }
}


fn data_from_extra_length_bits(symbol: HuffmanSymbol, extra_bits: u16) -> usize {
    let symbol = symbol as usize;
    let extra_bits = extra_bits as usize;

    match symbol {
        257..=264 => symbol - 254,
        265..=268 => 11 + (symbol - 265) << 1 + extra_bits,
        269..=272 => 19 + (symbol - 269) << 2 + extra_bits,
        273..=276 => 35 + (symbol - 273) << 3 + extra_bits,
        277..=280 => 67 + (symbol - 277) << 4 + extra_bits,
        281..=284 => 131 + (symbol - 281) << 5 + extra_bits,
        _ => 0 //This should never happen
    }
}

fn data_from_extra_dist_bits(symbol: HuffmanSymbol, extra_bits: u16) -> usize {
    let symbol = symbol as usize;
    let extra_bits = extra_bits as usize;

    match symbol {
        0..=3 => symbol + 1,
        4 | 5 => 5 + (symbol - 4) << 1 + extra_bits,
        6 | 7 => 9 + (symbol - 6) << 2 + extra_bits,
        8 | 9 => 17 + (symbol - 8) << 3 + extra_bits,
        10 | 11 => 33 + (symbol - 10) << 4 + extra_bits,
        12 | 13 => 65 + (symbol - 12) << 5 + extra_bits,
        14 | 15 => 129 + (symbol - 14) << 6 + extra_bits,
        16 | 17 => 257 + (symbol - 16) << 7 + extra_bits,
        18 | 19 => 513 + (symbol - 18) << 8 + extra_bits, 
        20 | 21 => 1025 + (symbol - 20) << 9 + extra_bits,
        22 | 23 => 2049 + (symbol - 22) << 10 + extra_bits,
        24 | 25 => 4097 + (symbol - 24) << 11 + extra_bits,
        26 | 27 => 8193 + (symbol - 26) << 12 + extra_bits,
        28 | 29 => 16385 + (symbol - 28) << 13 + extra_bits,
        _ => 0 //This should never happen
    }
}