use crate::bitstream::{BitReader, BitWriter};
use create::huffman::{encode_bytes_huffman, decode_bytes_huffman};

type MatchLength = i32;
type MatchOffset = i32;

fn fast_log2(v:u32) -> u32 {
    if (v == 0) {return 0;}

    31 - v.leading_zeros();
}

/// Given a length 
fn code_from_length(length: MatchLength) -> u32 {

}