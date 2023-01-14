use crate::bitstream::{BitReader, BitWriter};
use crate::huffman::{HuffmanSymbol, HuffmanPath, compress_huffman, decompress_huffman, HuffmanEncoder, HuffmanDecoder};
use std::collections::HashMap;
use std::fmt::{self};
use std::cmp::{min, max};

const CHUNK_SIZE:usize = 1 << 18;
const MAX_MATCH_NUM:usize = 16;

type LZLength = u32;
type LZOffset = u32;

fn fast_log2_floor_u32(n: u32) -> u32 {
    31 - n.leading_zeros()
}

fn huffman_symbol_from_length(length: usize) -> HuffmanSymbol {
    if length <= 15{
        return length as HuffmanSymbol;
    }

    (12 + fast_log2_floor_u32(length as u32)) as HuffmanSymbol
}

fn huffman_symbol_from_offset(offset: usize) -> HuffmanSymbol {
    if offset <= 2{
        return offset as HuffmanSymbol;
    }

    (1 + fast_log2_floor_u32(offset as u32)) as HuffmanSymbol
}

fn extra_huffman_symbol(v: usize) -> HuffmanSymbol {
    (v - (1 << fast_log2_floor_u32(v as u32))) as HuffmanSymbol
}

fn key_from_bytes(buffer: &[u8], pos: usize) -> u32{
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

pub struct LZEncoder {
    matcher: MatchFinder,
    do_optimal_parsing: bool,
    literals: Vec<u8>,
    match_lengths: Vec<usize>,
    match_offsets: Vec<usize>,
    literal_lengths: Vec<usize>
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
        let key = key_from_bytes(buffer, pos);

        if let Some(head) = self.head_map.get(&key){
            self.next_map.insert(pos, *head);
        }
        self.head_map.insert(key, pos);
    }

    fn find_match(&mut self, buffer:&[u8], pos: usize) -> (usize, usize) {
        let mut best_match_len:usize = 0;
        let mut best_match_pos:usize = 0;

        let key = key_from_bytes(buffer, pos);
        let min_pos_option:Option<usize> = if self.window_size > pos {None} else {Some(pos - self.window_size)};

        let mut next_option = self.head_map.get(&key);
        let mut hits = 0;
        let max_hits = 16;
        
        while let Some(next) = next_option {
            let next = *next;
            if let Some(min_pos) = min_pos_option {
                if next <= min_pos {break;}
            }
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

        //println!("Pos: {pos} Best match: {best_match_pos} Best match length; {best_match_len}");

        (best_match_len, best_match_pos)
    }

    fn find_matches(&mut self, buffer:&[u8], pos: usize) -> (Vec<usize>, Vec<usize>) {
        let mut match_lens = Vec::with_capacity(MAX_MATCH_NUM);
        let mut match_dists = Vec::with_capacity(MAX_MATCH_NUM);

        let key = key_from_bytes(buffer, pos);
        let min_pos_option:Option<usize> = if self.window_size > pos {None} else {Some(pos - self.window_size)};

        let mut next_option = self.head_map.get(&key);
        let mut hits = 0;
        
        while let Some(next) = next_option {
            let next = *next;
            if let Some(min_pos) = min_pos_option {
                if next <= min_pos {break;}
            }
            hits += 1;
            if hits >= MAX_MATCH_NUM {break;}

            let match_len = self.max_match_len(buffer, pos, next);
            if match_len > 0 {
                match_lens.push(match_len);
                match_dists.push(if next > pos {next - pos} else {pos - next});
            }

            next_option = self.next_map.get(&next);
        }

        if let Some(head) = self.head_map.get(&key){
            self.next_map.insert(pos, *head);
        }
        self.head_map.insert(key, pos);

        //println!("Pos: {pos} Best match: {best_match_pos} Best match length; {best_match_len}");

        (match_lens, match_dists)
    }


    fn max_match_len(&self, buffer: &[u8], source_pos: usize, match_pos: usize) -> usize {
        
        if key_from_bytes(buffer, source_pos) != key_from_bytes(buffer, match_pos) {
            return 0;
        }

        let mut len = 4;
        while source_pos + len < buffer.len() && buffer[source_pos + len] == buffer[match_pos + len] {
            len += 1;
        }

        len
    }
}

impl fmt::Display for LZEncoder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        let mut repr:String = String::new();
        repr.push_str("Literals: ");
        for byte in &self.literals{
            repr.push_str(format!("{} ",*byte).as_str());
        }
        repr.push_str("\nMatches:\n");
        for i in 0..self.match_lengths.len(){
            repr.push_str(format!("Match: [length: {} offset: {} literal_length: {}]\n", self.match_lengths[i], self.match_offsets[i], self.literal_lengths[i]).as_str());
        }

        write!(f,"{}",repr)
        
    }
}


impl LZEncoder {
    pub fn new(window_size: usize, do_optimal_parsing: bool) -> Self {
        LZEncoder { 
            matcher: MatchFinder::new(window_size), 
            do_optimal_parsing, 
            literals: Vec::with_capacity(CHUNK_SIZE), 
            match_lengths: Vec::with_capacity(CHUNK_SIZE >> 2), 
            match_offsets: Vec::with_capacity(CHUNK_SIZE >> 2), 
            literal_lengths: Vec::with_capacity(CHUNK_SIZE >> 2)
        }
    }

    fn simple_parse(&mut self, buffer: &[u8], start_pos: usize, end_pos: usize){
        let min_match_len:usize = 5;
        let mut literal_num = 0;
        let mut pos = start_pos;
    
        while pos < end_pos {
            let (mut match_len, match_pos) = self.matcher.find_match(buffer, pos);
            if match_len >= min_match_len && pos < end_pos - 4 {
    
                //println!("Pos: {pos} Match: [length: {match_len} offset: {} literal_count: {literal_num}]", pos - match_pos);
                
                self.match_lengths.push(match_len);
                self.match_offsets.push(pos - match_pos);
                self.literal_lengths.push(literal_num);
    
                literal_num = 0;
    
                match_len -= 1;
                while match_len > 0{
                    pos += 1;
                    self.matcher.insert(buffer, pos);
                    match_len -= 1;
                }
    
            } else {
                //println!("Pos: {pos} Literal: {}", buffer[pos]);
                self.literals.push(buffer[pos]);
                literal_num += 1;
            }
            pos += 1;
        }
    
        if literal_num > 0 {
            //println!("Match: [length: 0 offset: 0 literal_count: {literal_num}]");
            self.match_lengths.push(0);
            self.match_offsets.push(0);
            self.literal_lengths.push(literal_num);
        }
    
        //println!("Match lengths: {match_lengths:?}\nMatch offsets: {match_offsets:?}\nLiteral lengths: {literal_lengths:?}\nLiterals: {literals:?}");
    }

    fn optimal_parse_literal_price(byte: u8) -> u32 {6}

    ///
    /// 
    /// These costs were found using Glin Scott's tutorial. There might be better ones though
    fn optimal_parse_match_price(length: usize, offset: usize) -> u32{
        let length_cost = 6 + fast_log2_floor_u32(length as u32);
        let log2_dist = fast_log2_floor_u32(offset as u32);
        let offset_cost = if log2_dist >= 3 {log2_dist - 3} else {0};
        
        length_cost + offset_cost
    }

    fn optimal_parse(&mut self, buffer: &[u8], start_pos: usize, end_pos: usize) {
        let end_pos = min(buffer.len(), end_pos);
        let range = end_pos - start_pos;
        let mut matcher = MatchFinder::new(64);

        let mut prices:Vec<u32> = vec![u32::MAX; range + 1];
        let mut lengths:Vec<usize> = vec![0; range + 1];
        let mut offsets:Vec<usize> = vec![0; range + 1];

        prices[0] = 0;

        for i in 0..range {
            let literal_cost = prices[i] + LZEncoder::optimal_parse_literal_price(buffer[i]);
            if literal_cost < prices[i + 1] {
                prices[i + 1] = literal_cost;
                lengths[i + 1] = 1;
                offsets[i + 1] = 0;
            }

            if i + 4 >= range {continue;}

            let (match_lengths, match_dists) = matcher.find_matches(buffer, start_pos + i);
            for j in 0..match_lengths.len() {
                let match_price = prices[i] + LZEncoder::optimal_parse_match_price(match_lengths[j],match_dists[j]);
                if match_price < prices[i + match_lengths[j]] {
                    prices[i + match_lengths[j]] = match_price;
                    lengths[i + match_lengths[j]] = match_lengths[j];
                    offsets[i + match_lengths[j]] = match_dists[j];
                }
            }
        }

        if lengths[range] <= 1{
            let match_num = self.match_lengths.len();
            self.match_offsets.push(0);
            self.match_lengths.push(0);
            self.literal_lengths.push(0);
        }

        let mut i = range;
        while i > 0 {
            if lengths[i] > 1 {
                self.match_lengths.push(lengths[i]);
                self.match_offsets.push(offsets[i]);
                self.literal_lengths.push(0);
                i -= lengths[i];
            } else {
                self.literals.push(buffer[start_pos + i - 1]);
                self.literal_lengths[self.match_lengths.len() - 1] += 1;
                i -= 1;
            }
        }

        self.match_lengths = self.match_lengths.iter().copied().rev().collect();
        self.match_offsets = self.match_offsets.iter().copied().rev().collect();
        self.literal_lengths = self.literal_lengths.iter().copied().rev().collect();
        self.literals = self.literals.iter().copied().rev().collect();
    }

    pub fn parse(&mut self, buffer: &[u8], start_pos:usize, end_pos:usize) {
        self.literals.clear();
        self.match_lengths.clear();
        self.match_offsets.clear();
        self.literal_lengths.clear();
        match self.do_optimal_parsing{
            true => self.optimal_parse(buffer, start_pos, end_pos),
            false => self.simple_parse(buffer, start_pos, end_pos)
        }
    }

    fn huffman_encode_lengths(&self) -> Vec<u8> {
        let mut encoder:HuffmanEncoder = HuffmanEncoder::new(32);

        for i in 0..self.match_lengths.len() {
            encoder.scan_byte(huffman_symbol_from_length(self.match_lengths[i]));
        }

        encoder.build_huffman_table();
        
        for i in 0..self.match_lengths.len(){
            let length = self.match_lengths[i];
            encoder.encode_symbol(huffman_symbol_from_length(length));
            if length >= 16 {
                encoder.writer_mut().write_bits_u32(extra_huffman_symbol(length) as u32, fast_log2_floor_u32(length as u32) as usize);
            }
        }
        encoder.finish();

        encoder.writer().get_bytes()
    }

    fn huffman_encode_offsets(&self) -> Vec<u8> {
        let mut encoder:HuffmanEncoder = HuffmanEncoder::new(32);

        for i in 0..self.match_offsets.len() {
            encoder.scan_byte(huffman_symbol_from_length(self.match_offsets[i]));
        }

        encoder.build_huffman_table();
        
        for i in 0..self.match_offsets.len(){
            let offset = self.match_offsets[i];
            encoder.encode_symbol(huffman_symbol_from_offset(offset));
            if offset >= 2 {
                encoder.writer_mut().write_bits_u32(extra_huffman_symbol(offset) as u32, fast_log2_floor_u32(offset as u32) as usize);
            }
        }
        encoder.finish();

        encoder.writer().get_bytes()
    }

    fn huffman_encode_literal_lengths(&self) -> Vec<u8> {
        let mut encoder:HuffmanEncoder = HuffmanEncoder::new(32);

        for i in 0..self.literal_lengths.len() {
            encoder.scan_byte(huffman_symbol_from_length(self.literal_lengths[i]));
        }

        encoder.build_huffman_table();
        
        for i in 0..self.literal_lengths.len(){
            let literal_length = self.literal_lengths[i];
            encoder.encode_symbol(huffman_symbol_from_length(literal_length));
            if literal_length >= 16 {
                encoder.writer_mut().write_bits_u32(extra_huffman_symbol(literal_length) as u32, fast_log2_floor_u32(literal_length as u32) as usize);
            }
        }
        encoder.finish();

        encoder.writer().get_bytes()
    }

    fn huffman_encode_literals(&self) -> Vec<u8> {
        let mut encoder:HuffmanEncoder = HuffmanEncoder::new(256);

        encoder.writer_mut().write_bits_u32(self.literals.len() as u32, 32);
        encoder.build_frequency_table(&self.literals);
        encoder.build_huffman_table();
        encoder.encode_symbols(&self.literals);

        encoder.finish();
        encoder.writer().get_bytes()
    }

    pub fn huffman_encode_chunk(&mut self, buffer: &[u8], start_pos: usize, end_pos: usize) -> Vec<u8> {
        let mut encoded:Vec<u8> = Vec::new();
        self.parse(buffer, start_pos, end_pos);

        encoded.extend(self.huffman_encode_literals());
        encoded.extend(self.huffman_encode_lengths());
        encoded.extend(self.huffman_encode_offsets());
        encoded.extend(self.huffman_encode_literal_lengths());

        encoded
    }

    pub fn compress_huffman(&mut self, buffer: &[u8]) {
        let mut encoded:Vec<u8> = Vec::new();
        
    }


}

pub fn decompress_lz(encoded_bytes: &[u8]) -> Vec<u8>{
    let mut decoded_bytes = Vec::new();
    let mut reader = HuffmanDecoder::new(encoded_bytes);

    let mut literals = Vec::with_capacity(CHUNK_SIZE);
    let mut lengths = Vec::with_capacity(CHUNK_SIZE >> 2);
    let mut offsets = Vec::with_capacity(CHUNK_SIZE >> 2);
    let mut literal_offsets = Vec::with_capacity(CHUNK_SIZE >> 2);




    decoded_bytes
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
    fn lz_simple_parse_test() {
        use crate::lz::LZEncoder;
        use std::{fs, time};
        
        //let bytes = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
        let bytes = "TOBEORNOTTOBE".as_bytes().to_vec();
        let mut encoder:LZEncoder = LZEncoder::new(64, false);
        encoder.parse(&bytes, 0, bytes.len());
        println!("{encoder}");
    }

    #[test]
    fn lz_optimal_parse_test() {
        use crate::lz::LZEncoder;
        use std::{fs, time};
        
        //let bytes = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
        let bytes = "TOBEORNOTOBE".as_bytes().to_vec();
        let mut encoder:LZEncoder = LZEncoder::new(64, true);
        encoder.parse(&bytes, 0, bytes.len());
        println!("{encoder}");
    }
}