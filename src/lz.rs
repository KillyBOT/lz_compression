use crate::bitstream::{BitReader, BitWriter};
use crate::huffman::{HuffmanSymbol, HuffmanPath, compress_huffman, decompress_huffman};

type LZLength = u32;
type LZOffset = u32;

fn fast_log2_floor_u32(n: u32) -> u32 {
    31 - n.leading_zeros();
}

fn huffman_symbol_from_length(length: u32) -> HuffmanSymbol {
    if length <= 15{
        return length as HuffmanSymbol;
    }

    (12 + fast_log2_floor_u32(length)) as HuffmanSymbol
}

fn huffman_symbol_from_offset(offset: u32) -> HuffmanSymbol {
    if offset <= 2{
        return offset as HuffmanSymbol;
    }

    (1 + fast_log2_floor_u32(offset)) as HuffmanSymbol
}

fn extra_huffman_symbol_from_length(length: u32) -> HuffmanSymbol {
    (length - (1 << fast_log2_floor_u32(length))) as HuffmanSymbol
}

fn extra_huffman_symbol_from_length(offset: u32) -> HuffmanSymbol {
    (offset - (1 << fast_log2_floor_u32(offset))) as HuffmanSymbol
}

struct MatchFinder {
    window_size:usize,
    head_map:HashMap<usize, usize>,
    next_map:HashMap<u32, usize>
}

impl MatchFinder {
    fn new(window_size:usize) -> Self {
        MatchFinder {
            window_size,
            head_map: HashMap::with_capacity(window_size),
            next_map: HashMap::with_capacity(window_size)
        }
    }

    fn insert(&mut self, buffer:&[u8], pos: usize) {
        let key = u32::from_be_bytes(&buffer[pos..(pos+4)]);
        self.next_map.insert(pos, self.head_map.get(&key).unwrap_or(0));
        self.head_map.insert(key, pos);
    }

    fn find_match(&mut self, buffer:&[u8], pos: usize) -> (u32, u32) {
        let mut best_match_len:u32 = 0;
        let mut best_match_pos:u32 = 0;

        let key = u32::from_be_bytes(&buffer[pos..(pos + 4)]);
        let min_pos:isize = pos - self.window_size;
        let mut next = self.head_map[key];

        (0, 0)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fast_log2_floor_u32_test() {
        use rand::prelude::*;

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2123);
        let mut vals = Vec::with_capacity(byte_num);
        for _ in 0..byte_num {vals.push(rng.gen::<u32>());}

        for val in &vals {
            let val = *val;
            assert!(fast_log2_floor_u32(val) == (val as f32).log2().floor() as u32, "Fast log2 failed with value {val}");
        }
    }
}