// use crate::bitstream::{BitReader, BitWriter};
// use crate::huffman::{HuffmanSymbol, HuffmanPath, HuffmanEncoder, HuffmanDecoder, HUFFMAN_CHUNK_SIZE_BITS, HUFFMAN_MAX_SYMBOLS};
// use std::collections::HashMap;
// use std::fmt::{self};
// use std::cmp::{min, max};

// const LZ_CHUNK_SIZE:usize = 1 << 18;
// const MAX_MATCH_NUM:usize = 16;

// type LZLength = u32;
// type LZOffset = u32;

// fn fast_log2_floor_u32(n: u32) -> u32 {
//     31 - n.leading_zeros()
// }

// fn huffman_symbol_from_length(length: usize) -> HuffmanSymbol {
//     if length < 16{
//         return length as HuffmanSymbol;
//     }

//     (12 + fast_log2_floor_u32(length as u32)) as HuffmanSymbol
// }

// fn huffman_symbol_from_offset(offset: usize) -> HuffmanSymbol {
//     if offset < 2{
//         return offset as HuffmanSymbol;
//     }

//     (1 + fast_log2_floor_u32(offset as u32)) as HuffmanSymbol
// }

// fn extra_huffman_symbol(v: usize) -> HuffmanSymbol {
//     (v - (1 << fast_log2_floor_u32(v as u32))) as HuffmanSymbol
// }

// fn key_from_bytes(buffer: &[u8], pos: usize) -> u32{
//     let mut hash:u32 = 0;
//     let byte_num = if pos + 3 >= buffer.len() {buffer.len() - pos} else {3};
//     for i in 0..byte_num{
//         hash <<= 8;
//         hash |= buffer[pos + i] as u32;
//     }

//     hash
// }

// struct MatchFinder {
//     window_size:usize,
//     head_map:HashMap<u32, usize>,
//     next_map:HashMap<usize, usize>
// }

// pub struct LZEncoder<'a>{
//     writer: &'a mut BitWriter,
//     matcher: MatchFinder,
//     do_optimal_parsing: bool,
//     literals: Vec<u8>,
//     match_lengths: Vec<usize>,
//     match_offsets: Vec<usize>,
//     match_literal_lengths: Vec<usize>
// }

// pub struct LZDecoder<'a, 'b: 'a> {
//     decoder: HuffmanDecoder<'a, 'b>,
//     literals: Vec<u8>,
//     match_lengths:Vec<usize>,
//     match_offsets: Vec<usize>,
//     match_literal_lengths: Vec<usize>,
//     decoded: Vec<u8>
// }

// impl MatchFinder {
//     fn new(window_size:usize) -> Self {
//         MatchFinder {
//             window_size,
//             head_map: HashMap::with_capacity(window_size),
//             next_map: HashMap::with_capacity(window_size)
//         }
//     }

//     fn insert(&mut self, buffer:&[u8], pos: usize) {
//         let key = key_from_bytes(buffer, pos);

//         if let Some(head) = self.head_map.get(&key){
//             self.next_map.insert(pos, *head);
//         }
//         self.head_map.insert(key, pos);
//     }

//     fn find_match(&mut self, buffer:&[u8], pos: usize) -> (usize, usize) {
//         let mut best_match_len:usize = 0;
//         let mut best_match_pos:usize = 0;

//         let key = key_from_bytes(buffer, pos);
//         let min_pos_option:Option<usize> = if self.window_size > pos {None} else {Some(pos - self.window_size)};

//         let mut next_option = self.head_map.get(&key);
//         let mut hits = 0;
//         let max_hits = 16;
        
//         while let Some(next) = next_option {
//             let next = *next;
//             if let Some(min_pos) = min_pos_option {
//                 if next <= min_pos {break;}
//             }
//             hits += 1;
//             if hits >= max_hits {break;}

//             let match_len = self.max_match_len(buffer, pos, next);
//             if match_len > best_match_len {
//                 best_match_len = match_len;
//                 best_match_pos = next;
//             }

//             next_option = self.next_map.get(&next);
//         }

//         if let Some(head) = self.head_map.get(&key){
//             self.next_map.insert(pos, *head);
//         }
//         self.head_map.insert(key, pos);

//         //println!("Pos: {pos} Best match: {best_match_pos} Best match length; {best_match_len}");

//         (best_match_len, best_match_pos)
//     }

//     fn find_matches(&mut self, buffer:&[u8], pos: usize) -> (Vec<usize>, Vec<usize>) {
//         let mut match_lens = Vec::with_capacity(MAX_MATCH_NUM);
//         let mut match_dists = Vec::with_capacity(MAX_MATCH_NUM);

//         let key = key_from_bytes(buffer, pos);
//         let min_pos_option:Option<usize> = if self.window_size > pos {None} else {Some(pos - self.window_size)};

//         let mut next_option = self.head_map.get(&key);
//         let mut hits = 0;
        
//         while let Some(next) = next_option {
//             let next = *next;
//             if let Some(min_pos) = min_pos_option {
//                 if next <= min_pos {break;}
//             }
//             hits += 1;
//             if hits >= MAX_MATCH_NUM {break;}

//             let match_len = self.max_match_len(buffer, pos, next);
//             if match_len > 0 {
//                 match_lens.push(match_len);
//                 match_dists.push(if next > pos {next - pos} else {pos - next});
//             }

//             next_option = self.next_map.get(&next);
//         }

//         if let Some(head) = self.head_map.get(&key){
//             self.next_map.insert(pos, *head);
//         }
//         self.head_map.insert(key, pos);

//         //println!("Pos: {pos} Best match: {best_match_pos} Best match length; {best_match_len}");

//         (match_lens, match_dists)
//     }


//     fn max_match_len(&self, buffer: &[u8], source_pos: usize, match_pos: usize) -> usize {
        
//         if key_from_bytes(buffer, source_pos) != key_from_bytes(buffer, match_pos) {
//             return 0;
//         }

//         let mut len = 4;
//         while source_pos + len < buffer.len() && buffer[source_pos + len] == buffer[match_pos + len] {
//             len += 1;
//         }

//         len
//     }
// }

// impl<'a> fmt::Display for LZEncoder<'a>{
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

//         let mut repr:String = String::new();
//         repr.push_str("Literals: ");
//         for byte in &self.literals{
//             repr.push_str(format!("{} ",*byte).as_str());
//         }
//         repr.push_str("\nMatches:\n");
//         for i in 0..self.match_lengths.len(){
//             repr.push_str(format!("Match: [length: {} offset: {} literal_length: {}]\n", self.match_lengths[i], self.match_offsets[i], self.match_literal_lengths[i]).as_str());
//         }

//         write!(f,"{}",repr)
        
//     }
// }


// impl<'a> LZEncoder<'a>{
//     pub fn new(writer: &'a mut BitWriter, window_size: usize, do_optimal_parsing: bool) -> Self {
//         LZEncoder { 
//             writer,
//             matcher: MatchFinder::new(window_size), 
//             do_optimal_parsing, 
//             literals: Vec::with_capacity(LZ_CHUNK_SIZE), 
//             match_lengths: Vec::with_capacity(LZ_CHUNK_SIZE >> 2), 
//             match_offsets: Vec::with_capacity(LZ_CHUNK_SIZE >> 2), 
//             match_literal_lengths: Vec::with_capacity(LZ_CHUNK_SIZE >> 2)
//         }
//     }

//     fn simple_parse(&mut self, buffer: &[u8]){
//         let min_match_len:usize = 5;
//         let mut literal_num = 0;
//         let mut pos = 0;
    
//         while pos < buffer.len() {
//             let (mut match_len, match_pos) = self.matcher.find_match(buffer, pos);
//             if match_len >= min_match_len && pos < buffer.len() - 4 {
    
//                 //println!("Pos: {pos} Match: [length: {match_len} offset: {} literal_count: {literal_num}]", pos - match_pos);
                
//                 self.match_lengths.push(match_len);
//                 self.match_offsets.push(pos - match_pos);
//                 self.match_literal_lengths.push(literal_num);
    
//                 literal_num = 0;
    
//                 match_len -= 1;
//                 while match_len > 0{
//                     pos += 1;
//                     self.matcher.insert(buffer, pos);
//                     match_len -= 1;
//                 }
    
//             } else {
//                 //println!("Pos: {pos} Literal: {}", buffer[pos]);
//                 self.literals.push(buffer[pos]);
//                 literal_num += 1;
//             }
//             pos += 1;
//         }
    
//         if literal_num > 0 {
//             //println!("Match: [length: 0 offset: 0 literal_count: {literal_num}]");
//             self.match_lengths.push(0);
//             self.match_offsets.push(0);
//             self.match_literal_lengths.push(literal_num);
//         }
    
//         //println!("Match lengths: {match_lengths:?}\nMatch offsets: {match_offsets:?}\nLiteral lengths: {literal_lengths:?}\nLiterals: {literals:?}");
//     }

//     fn optimal_parse_literal_price(byte: u8) -> u32 {6}

//     ///
//     /// 
//     /// These costs were found using Glin Scott's tutorial. There might be better ones though
//     fn optimal_parse_match_price(length: usize, offset: usize) -> u32{
//         let length_cost = 6 + fast_log2_floor_u32(length as u32);
//         let log2_dist = fast_log2_floor_u32(offset as u32);
//         let offset_cost = if log2_dist >= 3 {log2_dist - 3} else {0};
        
//         length_cost + offset_cost
//     }

//     fn optimal_parse(&mut self, buffer: &[u8]) {
//         let mut matcher = MatchFinder::new(64);

//         let mut prices:Vec<u32> = vec![u32::MAX; buffer.len() + 1];
//         let mut lengths:Vec<usize> = vec![0; buffer.len() + 1];
//         let mut offsets:Vec<usize> = vec![0; buffer.len() + 1];

//         prices[0] = 0;

//         for i in 0..buffer.len() {
//             let literal_cost = prices[i] + LZEncoder::optimal_parse_literal_price(buffer[i]);
//             if literal_cost < prices[i + 1] {
//                 prices[i + 1] = literal_cost;
//                 lengths[i + 1] = 1;
//                 offsets[i + 1] = 0;
//             }

//             if i + 4 >= buffer.len() {continue;}

//             let (match_lengths, match_dists) = matcher.find_matches(buffer, i);
//             for j in 0..match_lengths.len() {
//                 let match_price = prices[i] + LZEncoder::optimal_parse_match_price(match_lengths[j],match_dists[j]);
//                 if match_price < prices[i + match_lengths[j]] {
//                     prices[i + match_lengths[j]] = match_price;
//                     lengths[i + match_lengths[j]] = match_lengths[j];
//                     offsets[i + match_lengths[j]] = match_dists[j];
//                 }
//             }
//         }

//         if lengths[buffer.len()] <= 1{
//             let match_num = self.match_lengths.len();
//             self.match_offsets.push(0);
//             self.match_lengths.push(0);
//             self.match_literal_lengths.push(0);
//         }

//         let mut i = buffer.len();
//         while i > 0 {
//             if lengths[i] > 1 {
//                 self.match_lengths.push(lengths[i]);
//                 self.match_offsets.push(offsets[i]);
//                 self.match_literal_lengths.push(0);
//                 i -= lengths[i];
//             } else {
//                 self.literals.push(buffer[i - 1]);
//                 self.match_literal_lengths[self.match_lengths.len() - 1] += 1;
//                 i -= 1;
//             }
//         }

//         self.match_lengths = self.match_lengths.iter().copied().rev().collect();
//         self.match_offsets = self.match_offsets.iter().copied().rev().collect();
//         self.match_literal_lengths = self.match_literal_lengths.iter().copied().rev().collect();
//         self.literals = self.literals.iter().copied().rev().collect();
//     }

//     pub fn parse(&mut self, buffer: &[u8]) {
//         self.literals.clear();
//         self.match_lengths.clear();
//         self.match_offsets.clear();
//         self.match_literal_lengths.clear();
//         match self.do_optimal_parsing{
//             true => self.optimal_parse(buffer),
//             false => self.simple_parse(buffer)
//         }
//     }

//     fn huffman_encode_lengths(&mut self) {
//         let mut encoder:HuffmanEncoder = HuffmanEncoder::new(self.writer, 32);

//         for i in 0..self.match_lengths.len() {
//             encoder.scan_symbol(huffman_symbol_from_length(self.match_lengths[i]));
//         }

//         encoder.build_huffman_table();
//         encoder.writer.write_bits_u32(self.match_lengths.len() as u32, HUFFMAN_CHUNK_SIZE_BITS);
        
//         for i in 0..self.match_lengths.len(){
//             let length = self.match_lengths[i];
//             encoder.encode_symbol(huffman_symbol_from_length(length));
//             if length >= 16 {
//                 encoder.writer.write_bits_u32(extra_huffman_symbol(length) as u32, fast_log2_floor_u32(length as u32) as usize);
//             }
//         }
//     }

//     fn huffman_encode_offsets(&mut self) {
//         let mut encoder:HuffmanEncoder = HuffmanEncoder::new(self.writer, 32);

//         for i in 0..self.match_offsets.len() {
//             encoder.scan_symbol(huffman_symbol_from_offset(self.match_offsets[i]));
//         }

//         encoder.build_huffman_table();

//         encoder.writer.write_bits_u32(self.match_offsets.len() as u32, HUFFMAN_CHUNK_SIZE_BITS);
//         for i in 0..self.match_offsets.len(){
//             let offset = self.match_offsets[i];
//             encoder.encode_symbol(huffman_symbol_from_offset(offset));
//             if offset >= 2 {
//                 encoder.writer.write_bits_u32(extra_huffman_symbol(offset) as u32, fast_log2_floor_u32(offset as u32) as usize);
//             }
//         }

//     }

//     fn huffman_encode_literal_lengths(&mut self){
//         let mut encoder:HuffmanEncoder = HuffmanEncoder::new(self.writer, 32);

//         for i in 0..self.match_literal_lengths.len() {
//             encoder.scan_symbol(huffman_symbol_from_length(self.match_literal_lengths[i]));
//         }

//         encoder.build_huffman_table();

//         encoder.writer.write_bits_u32(self.match_literal_lengths.len() as u32, HUFFMAN_CHUNK_SIZE_BITS);
//         for i in 0..self.match_literal_lengths.len(){
//             let literal_length = self.match_literal_lengths[i];
//             encoder.encode_symbol(huffman_symbol_from_length(literal_length));
//             if literal_length >= 16 {
//                 encoder.writer.write_bits_u32(extra_huffman_symbol(literal_length) as u32, fast_log2_floor_u32(literal_length as u32) as usize);
//             }
//         }


//     }

//     fn huffman_encode_literals(&mut self){
//         let mut encoder:HuffmanEncoder = HuffmanEncoder::new(self.writer, HUFFMAN_MAX_SYMBOLS);

//         encoder.encode_all_bytes(&self.literals, usize::MAX);
//     }

//     pub fn huffman_encode_chunk(&mut self, buffer: &[u8]){
//         self.parse(buffer);

//         self.huffman_encode_literals();
//         self.huffman_encode_lengths();
//         self.huffman_encode_offsets();
//         self.huffman_encode_literal_lengths();
//     }

//     pub fn huffman_encode_all(&mut self, buffer: &[u8], chunk_size: usize) {
//         let chunk_size = min(chunk_size, buffer.len());

//         for start_pos in (0..buffer.len()).step_by(chunk_size){
//             let end_pos = min(start_pos + chunk_size, buffer.len());
//             let chunk = &buffer[start_pos..end_pos];
//             self.huffman_encode_chunk(chunk);
//         }
//     }

//     pub fn writer(&self) -> &BitWriter {
//         &self.writer
//     }

//     pub fn writer_mut(&mut self) -> &mut BitWriter {
//         &mut self.writer
//     }

// }

// impl<'a, 'b:'a> LZDecoder<'a, 'b> {
//     pub fn new(reader: &'a mut BitReader<'b>) -> Self {
//         LZDecoder { 
//             decoder: HuffmanDecoder::new(reader), 
//             literals: Vec::with_capacity(LZ_CHUNK_SIZE), 
//             match_lengths: Vec::with_capacity(LZ_CHUNK_SIZE >> 2),
//             match_offsets: Vec::with_capacity(LZ_CHUNK_SIZE >> 2), 
//             match_literal_lengths: Vec::with_capacity(LZ_CHUNK_SIZE >> 2),
//             decoded: Vec::new()
//         }
//     }

//     fn huffman_decode_literals(&mut self) {
//         self.literals.clear();

//         self.decoder.read_huffman_table();
//         self.literals.append(&mut HuffmanDecoder::symbols_to_bytes(&self.decoder.decode_chunk()));
//         //println!("Literals: {:?}", self.literals);
//     }

//     fn huffman_decode_lengths(&mut self) {
//         self.match_lengths.clear();

//         self.decoder.read_huffman_table();
//         let match_num = self.decoder.reader.read_bits_into_u32(HUFFMAN_CHUNK_SIZE_BITS).unwrap();
//         for _ in 0..match_num{
//             let mut val = self.decoder.decode_one() as u32;
//             if val >= 16 {
//                 let extra_bits = val - 12;
//                 val = (1 << extra_bits) | self.decoder.reader.read_bits_into_u32(extra_bits as usize).unwrap();
//             }
//             self.match_lengths.push(val as usize);
//         }

//         //println!("Match lengths: {:?}", self.match_lengths);
//     }

//     fn huffman_decode_offsets(&mut self) {
//         self.match_offsets.clear();

//         self.decoder.read_huffman_table();
//         let match_num = self.decoder.reader.read_bits_into_u32(HUFFMAN_CHUNK_SIZE_BITS).unwrap();
//         for _ in 0..match_num{
//             let mut val = self.decoder.decode_one() as u32;
//             if val >= 2 {
//                 let extra_bits = val - 1;
//                 val = (1 << extra_bits) | self.decoder.reader.read_bits_into_u32(extra_bits as usize).unwrap();
//             }
//             self.match_offsets.push(val as usize);
//         }

//         //println!("Match offsets: {:?}", self.match_offsets);
//     }

//     fn huffman_decode_literal_lengths(&mut self) {
//         self.match_literal_lengths.clear();

//         self.decoder.read_huffman_table();
//         let match_num = self.decoder.reader.read_bits_into_u32(HUFFMAN_CHUNK_SIZE_BITS).unwrap();
//         for _ in 0..match_num{
//             let mut val = self.decoder.decode_one() as u32;
//             if val >= 16 {
//                 let extra_bits = val - 12;
//                 val = (1 << extra_bits) | self.decoder.reader.read_bits_into_u32(extra_bits as usize).unwrap();
//             }
//             self.match_literal_lengths.push(val as usize);
//         }

//         //println!("Match literal lengths: {:?}", self.match_literal_lengths);
//     }

//     pub fn huffman_decode_chunk(&mut self) -> Vec<u8>{
//         let mut decoded = Vec::new();
//         self.huffman_decode_literals();
//         self.huffman_decode_lengths();
//         self.huffman_decode_offsets();
//         self.huffman_decode_literal_lengths();

//         let mut curr_literal:usize = 0;
//         for i in 0..self.match_lengths.len(){
//             for _ in 0..self.match_literal_lengths[i]{
//                 decoded.push(self.literals[curr_literal]);
//                 curr_literal += 1;
//             }
//             let match_start = decoded.len() - self.match_offsets[i];

//             for j in 0..self.match_lengths[i] {
//                 decoded.push(decoded[match_start + j]);
//             }
//         }

//         decoded
//     }

//     pub fn huffman_decode_all(&mut self) -> Vec<u8> {
//         let mut decoded = Vec::new();
//         while self.decoder.reader.remaining_bits() > HUFFMAN_CHUNK_SIZE_BITS {
//             decoded.append(&mut self.huffman_decode_chunk());
//         }

//         decoded
//     }

// }

// #[cfg(test)]
// mod tests {
//     use crate::{bitstream::{BitWriter, BitReader}, lz::{LZDecoder, LZ_CHUNK_SIZE}};

//     #[test]
//     fn fast_log2_floor_u32_test() {
//         use rand::prelude::*;
//         use crate::lz::fast_log2_floor_u32;

//         let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2123);
//         let mut vals = Vec::with_capacity(8192);
//         for _ in 0..8192 {vals.push(rng.gen::<u32>());}

//         for val in &vals {
//             let val = *val;
//             assert!(fast_log2_floor_u32(val) == (val as f32).log2().floor() as u32, "Fast log2 failed with value {val}");
//         }
//     }

//     #[test]
//     fn lz_simple_parse_test() {
//         use crate::lz::LZEncoder;
//         use std::{fs, time};
        
//         let bytes = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
//         let mut writer = BitWriter::new();
//         let mut encoder:LZEncoder = LZEncoder::new(&mut writer, 64, false);

//         let start_time = time::Instant::now();

//         encoder.parse(&bytes);

//         let elapsed_time = start_time.elapsed().as_millis();
//         println!("Simple parse took {elapsed_time}ms at a speed of {}MB/s", ((bytes.len() as f32) / 1000000f32) / ((elapsed_time as f32) / 1000f32));

//         //println!("{encoder}");
//     }

//     #[test]
//     fn lz_optimal_parse_test() {
//         use crate::lz::LZEncoder;
//         use std::{fs, time};
        
//         let bytes = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
//         let mut writer = BitWriter::new();
//         let mut encoder:LZEncoder = LZEncoder::new(&mut writer, 64, true);

//         let start_time = time::Instant::now();

//         encoder.parse(&bytes);

//         let elapsed_time = start_time.elapsed().as_millis();
//         println!("Simple parse took {elapsed_time}ms at a speed of {}MB/s", ((bytes.len() as f32) / 1000000f32) / ((elapsed_time as f32) / 1000f32));

//     }
//     #[test]
//     fn lz_compression_decompression_test() {
//         use crate::lz::{LZEncoder};
//         use std::{fs, time};

//         //let contents = "ABCABCABCDEDEGGZ".as_bytes().to_vec();
//         let contents = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
//         let mut writer = BitWriter::new();
//         let mut encoder:LZEncoder = LZEncoder::new(&mut writer, 64, true);
        
//         let start_time = time::Instant::now();
//         encoder.huffman_encode_all(&contents, LZ_CHUNK_SIZE);
//         let encoded_bytes = encoder.writer().get_bytes();

//         let elapsed_time = start_time.elapsed().as_millis();
//         println!("Bytes unencoded: [{}] Bytes encoded:[{}] Compression ratio:[{}]\nTime:[{}]ms Speed:[{}]MB/s",contents.len(), encoded_bytes.len(), (encoded_bytes.len() as f32) / (contents.len() as f32), elapsed_time, ((contents.len() as f32) / 1000f32) / (elapsed_time as f32));
        
        
//         let mut reader = BitReader::new(&encoded_bytes);
//         let mut decoder = LZDecoder::new(&mut reader);
//         let start_time = time::Instant::now();
//         let decoded_bytes = decoder.huffman_decode_all();
//         let elapsed_time = start_time.elapsed().as_millis();
//         println!("Decompression time:[{}]ms Speed:[{}]MB/s", elapsed_time, ((encoded_bytes.len() as f32) / 1000f32) / (elapsed_time as f32));

//         assert!(contents.len() == decoded_bytes.len(), "Number of bytes different after encoding and decoding");
//         for i in 0..contents.len(){
//             assert!(contents[i] == decoded_bytes[i], "Byte at position {i} different after encoding and decoding [{}] -> [{}]", contents[i], decoded_bytes[i]);
//         }
        
//     }
// }