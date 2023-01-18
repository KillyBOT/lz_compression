use crate::bitstream::{BitReader, BitWriter};
use std::collections::HashMap;
use std::fmt::{self};
use std::cmp::{min, max};

const MAX_MATCH_NUM:usize = 16;
const MIN_MATCH_LEN:usize = 4;

type LZ77MapKey = u32;

struct LZ77MatchFinder {
    window_size:usize,
    min_match_len:usize,
    head_map:HashMap<u32, usize>,
    next_map:HashMap<usize, usize>
}

#[derive(Debug)]
pub struct LZ77Match {
    length: usize,
    offset: usize
}
#[derive(Debug)]
pub struct LZ77Encoded {
    literals: Vec<u8>,
    matches: Vec<LZ77Match>
}

fn key_from_bytes(buffer: &[u8], pos: usize) -> LZ77MapKey{
    let mut hash:LZ77MapKey = 0;
    let byte_num = min(buffer.len() - pos, 3);

    for i in 0..byte_num{
        hash <<= 8;
        hash |= buffer[pos + i] as LZ77MapKey;
    }

    hash
}

impl LZ77Match {
    pub fn length(&self) -> usize { self.length; }
    pub fn offset(&self) -> usize { self.offset; }
}

impl LZ77MatchFinder {
    fn new(window_size:usize, min_match_len:usize) -> Self {
        LZ77MatchFinder {
            window_size,
            min_match_len,
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
        let mut length:usize = 0;
        let mut offset:usize = 0;

        let min_pos:usize = if self.window_size > pos {0} else {pos - self.window_size};
        let mut next_option = self.head_map.get(&key);

        let mut matches = 0;
        
        while let Some(next) = next_option {
            let next = *next;
            if next < min_pos {break;}
            matches += 1;
            if matches >= MAX_MATCH_NUM {break;}

            let match_len = self.max_match_len(buffer, pos, next);
            if match_len > length {
                length = match_len;
                offset = pos - next;
            }

            next_option = self.next_map.get(&next);
        }

        self.insert(pos);

        //println!("Pos: {pos} Best match: {best_match_pos} Best match length; {best_match_len}");

        LZ77Match { length, offset }
    }

    fn find_matches(&mut self, buffer:&[u8], pos: usize) -> Vec<LZ77Match> {
        let mut matches = Vec::with_capacity(MAX_MATCH_NUM);

        let min_pos:usize = if self.window_size > pos {0} else {pos - self.window_size};
        let mut next_option = self.head_map.get(&key);
        let mut matches = 0;
        
        while let Some(next) = next_option {
            let next = *next;

            if next < min_pos {break;}

            matches += 1;
            if matches >= MAX_MATCH_NUM {break;}

            let length = self.max_match_len(buffer, pos, next);
            let offset = pos - next;

            if match_len > 0 {
                matches.push(LZ77Match{ length, offset });
            }

            next_option = self.next_map.get(&next);
        }

        self.insert(pos);

        //println!("Pos: {pos} Best match: {best_match_pos} Best match length; {best_match_len}");

        matches
    }


    fn max_match_len(&self, buffer: &[u8], source_pos: usize, match_pos: usize) -> usize {

        let mut len = 0;
        while source_pos + len < buffer.len() && buffer[source_pos + len] == buffer[match_pos + len] {
            len += 1;
        }

        if len >= MIN_MATCH_LEN {len} else {0}
    }
}

pub fn lz77_parse_simple(buffer: &[u8], window_size: usize, min_match_len: usize) -> LZ77Encoded{
    let mut matcher: LZ77MatchFinder = LZ77MatchFinder::new(window_size, min_match_len);
    let mut matches = Vec::new();
    let mut literals = Vec::with_capacity(buffer.len());
    
    let mut length = 0;
    let mut literal_num = 0;
    let mut pos = 0;

    while pos < buffer.len() {
        let lz77_match = matcher.find_match(buffer, pos);
        let mut length = lz77_match.length();
        if length >= min_match_len {

            //println!("Pos: {pos} Match: [length: {match_len} offset: {} literal_count: {literal_num}]", pos - match_pos);
            
            matches.push(lz77_match);
            self.match_literal_lengths.push(literal_num);

            literal_num = 0;
            match_len -= 1;

            while match_len > 0{
                pos += 1;
                matcher.insert(buffer, pos);
                match_len -= 1;
            }
        } else {
            //println!("Pos: {pos} Literal: {}", buffer[pos]);
            literals.push(buffer[pos]);
            literal_num += 1;
        }

        pos += 1;
    }

    if literal_num > 0 {
        //println!("Match: [length: 0 offset: 0 literal_count: {literal_num}]");
        self.match_lengths.push(0);
        self.match_offsets.push(0);
        self.match_literal_lengths.push(literal_num);
    }

    LZ77Encoded {literals, matches}

    //println!("Match lengths: {match_lengths:?}\nMatch offsets: {match_offsets:?}\nLiteral lengths: {literal_lengths:?}\nLiterals: {literals:?}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn lz77_parse_simple(){
        use crate::lz77::{lz77_parse_simple, };
        use std::{fs, time};
        
        //let bytes = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
        let bytes = "Blah blah blah!".as_bytes().to_vec();

        let start_time = time::Instant::now();
        let lz77_encoded = lzyy_parse_simple(&bytes, 32 * 1024, 4);
        let elapsed_time = start_time.elapsed().as_millis();
        println!("Simple parse took {elapsed_time}ms at a speed of {}MB/s", ((bytes.len() as f32) / 1000000f32) / ((elapsed_time as f32) / 1000f32));

        //println!("{encoder}");
    }
}