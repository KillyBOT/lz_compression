use crate::bitstream::{BitReader, BitWriter};
use crate::huffman::{HuffmanSymbol, HuffmanPath, compress_huffman, decompress_huffman};
use std::collections::HashMap;

type LZLength = u32;
type LZOffset = u32;

fn fast_log2_floor_u32(n: u32) -> u32 {
    31 - n.leading_zeros()
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

fn extra_huffman_symbol_from_offset(offset: u32) -> HuffmanSymbol {
    (offset - (1 << fast_log2_floor_u32(offset))) as HuffmanSymbol
}

fn hash_from_bytes(buffer: &[u8], pos: usize) -> u32{
    let mut hash:u32 = 0;
    let byte_num = if pos + 4 >= buffer.len() {buffer.len() - pos} else {4};
    for i in 0..byte_num{
        hash <<= 8;
        hash |= buffer[pos + i] as u32;
    }

    hash
}

struct MatchFinder {
    window_size:usize,
    head_map:HashMap<u32, usize>,
    next_map:HashMap<usize, usize>
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
        let key = hash_from_bytes(buffer, pos);

        if let Some(head) = self.head_map.get(&key){
            self.next_map.insert(pos, *head);
        }
        self.head_map.insert(key, pos);
    }

    fn find_match(&mut self, buffer:&[u8], pos: usize) -> (usize, usize) {
        let mut best_match_len:usize = 0;
        let mut best_match_pos:usize = 0;

        let key = hash_from_bytes(buffer, pos);
        let min_pos = if self.window_size > pos {0} else {pos - self.window_size};

        let mut next_option = self.head_map.get(&key);
        let mut hits = 0;
        let max_hits = 16;
        
        while let Some(next) = next_option {
            let next = *next;
            if next <= min_pos {break;}
            hits += 1;
            if hits >= max_hits {break;}

            let match_len = self.max_match_len(buffer, pos, next);
            if match_len > best_match_len {
                best_match_len = match_len;
                best_match_pos = next;
            }

            next_option = self.next_map.get(&next);
        }

        if let Some(head) = self.head_map.get(&key){
            self.next_map.insert(pos, *head);
        }
        self.head_map.insert(key, pos);


        (best_match_len, best_match_pos)
    }

    fn max_match_len(&self, buffer: &[u8], source_pos: usize, match_pos: usize) -> usize {
        
        if hash_from_bytes(buffer, source_pos) != hash_from_bytes(buffer, match_pos) {
            return 0;
        }

        let mut len = 4;
        while source_pos + len < buffer.len() && buffer[source_pos + len] == buffer[match_pos + len] {
            len += 1;
        }

        len
    }
}

fn lz_simple_parse(buffer: &[u8], start_pos: usize, end_pos: usize){
    let min_match_len:usize = 5;
    let mut total_literal_num = 0;
    let mut sequence_num = 0;
    let mut literal_num = 0;
    let mut finder = MatchFinder::new(64);
    let mut literals: Vec<u8> = Vec::new();
    let mut literal_lengths: Vec<usize> = Vec::new();
    let mut match_lengths: Vec<usize> = Vec::new();
    let mut match_offsets: Vec<usize> = Vec::new();

    let mut pos = start_pos;

    while pos < end_pos {
        let (mut match_len, match_pos) = finder.find_match(buffer, pos);
        if match_len >= min_match_len && pos < end_pos - 4 {
            match_lengths.push(match_len);
            match_offsets.push(pos - match_pos);
            literal_lengths.push(literal_num);

            literal_num = 0;
            sequence_num += 1;

            match_len -= 1;
            while match_len > 0{
                pos += 1;
                finder.insert(buffer, pos);
                match_len -= 1;
            }

        } else {
            literals.push(buffer[pos]);
            literal_num += 1;
            total_literal_num += 1;
        }
        pos += 1;
    }

    if literal_num > 0 {
        match_lengths.push(0);
        match_offsets.push(0);
        literal_lengths.push(literal_num);
    }

    println!("Match lengths: {match_lengths:?}\nMatch offsets: {match_offsets:?}\nLiteral lengths: {literal_lengths:?}\nLiterals: {literals:?}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn fast_log2_floor_u32_test() {
        use rand::prelude::*;
        use crate::lz::fast_log2_floor_u32;

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2123);
        let mut vals = Vec::with_capacity(8192);
        for _ in 0..8192 {vals.push(rng.gen::<u32>());}

        for val in &vals {
            let val = *val;
            assert!(fast_log2_floor_u32(val) == (val as f32).log2().floor() as u32, "Fast log2 failed with value {val}");
        }
    }

    #[test]
    fn simple_match_finder_test() {
        use crate::lz::lz_simple_parse;

        use std::{fs, time};
        
        //let bytes = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
        let bytes = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Consequat nisl vel pretium lectus. Tempor orci dapibus ultrices in iaculis nunc sed augue. Mattis nunc sed blandit libero volutpat sed. Dictumst vestibulum rhoncus est pellentesque. Est lorem ipsum dolor sit amet consectetur adipiscing. Cursus sit amet dictum sit amet justo donec enim. Auctor augue mauris augue neque gravida. Fames ac turpis egestas integer eget aliquet nibh. Interdum varius sit amet mattis. Et netus et malesuada fames. Tellus at urna condimentum mattis pellentesque. Eros donec ac odio tempor orci dapibus. Quam vulputate dignissim suspendisse in.
        ".as_bytes().to_vec();
        lz_simple_parse(&bytes, 0, bytes.len());
    }
}