use crate::bitstream::{BitReader, BitWriter};
use std::collections::HashMap;
use std::fmt::{self};
use std::cmp::{min, max};

const LZ77_WINDOW_SIZE:usize = 32 * 1024;

pub struct LZ77Encoder {
    window_size:usize,
    head_map:HashMap<u32, usize>,
    next_map:HashMap<usize, usize>
}

impl LZ77Encoder {
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
