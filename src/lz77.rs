use std::collections::HashMap;
use std::fmt::{self};
use std::cmp::{min};

const MAX_MATCH_NUM:usize = 16;
const MIN_MATCH_LEN:usize = 4;

type LZ77MapKey = u32;
struct LZ77MatchFinder {
    window_size:usize,
    min_match_len:usize,
    head_map:HashMap<u32, usize>,
    next_map:HashMap<usize, usize>
}

#[derive(Clone, Copy)]
pub enum LZ77Data {
    Literal(u8),
    Match(usize, usize)

}

pub struct LZ77Encoded {
    data: Vec<LZ77Data>
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

impl LZ77MatchFinder {
    fn new(window_size:usize, min_match_len:usize) -> Self {

        assert!(min_match_len > 0, "Minimum match length cannot be 0!");
        assert!(window_size > 0, "Window size must be greater than 1!");

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

    fn find_match(&mut self, buffer:&[u8], pos: usize) -> LZ77Data {
        let mut length:usize = 0;
        let mut offset:usize = 0;

        let min_pos:usize = if self.window_size > pos {0} else {pos - self.window_size};
        let mut next_option = self.head_map.get(&key_from_bytes(buffer, pos));
        let mut match_num = 0;
        
        while let Some(next) = next_option {
            let next = *next;
            if next < min_pos {break;}
            match_num += 1;
            if match_num >= MAX_MATCH_NUM {break;}

            let match_len = self.max_match_len(buffer, pos, next);
            if match_len > length {
                length = match_len;
                offset = pos - next;
            }

            next_option = self.next_map.get(&next);
        }

        self.insert(buffer, pos);

        //println!("Pos: {pos} Best match: {best_match_pos} Best match length; {best_match_len}");

        if length >= self.min_match_len && offset >= self.min_match_len {LZ77Data::Match(length, offset)} else {LZ77Data::Literal(buffer[pos])}
    }

    fn find_matches(&mut self, buffer:&[u8], pos: usize) -> Vec<LZ77Data> {
        let mut data = Vec::with_capacity(MAX_MATCH_NUM);

        let min_pos:usize = if self.window_size > pos {0} else {pos - self.window_size};
        let mut next_option = self.head_map.get(&key_from_bytes(buffer, pos));
        let mut match_num = 0;
        
        while let Some(next) = next_option {
            let next = *next;
            if next < min_pos {break;}

            match_num += 1;
            if match_num >= MAX_MATCH_NUM {break;}

            let length = self.max_match_len(buffer, pos, next);

            if length >= self.min_match_len {
                data.push(LZ77Data::Match(length, pos - next));
            }

            next_option = self.next_map.get(&next);
        }

        self.insert(buffer, pos);

        //println!("Pos: {pos} Best match: {best_match_pos} Best match length; {best_match_len}");

        data
    }


    fn max_match_len(&self, buffer: &[u8], source_pos: usize, match_pos: usize) -> usize {

        let mut len = 0;
        let mut dist = source_pos - match_pos;

        while source_pos + len < buffer.len() && buffer[source_pos + len] == buffer[match_pos + (len % dist)] {
            len += 1;
        }

        len
    }
}

pub fn lz77_compress_simple(buffer: &[u8], window_size: usize, min_match_len: usize) -> LZ77Encoded{
    let mut matcher: LZ77MatchFinder = LZ77MatchFinder::new(window_size, min_match_len);
    let mut data = Vec::with_capacity(buffer.len());
    let mut pos = 0;

    while pos < buffer.len() {
        //println!("\r{pos} {} {}", buffer.len(), (pos as f32) / (buffer.len() as f32));

        let d = matcher.find_match(buffer, pos);
        data.push(d);

        match d {
            LZ77Data::Match(length, _) => {
                //println!("Found match of length {length}");
                let mut length = length - 1;
                while length > 0 {
                    pos += 1;
                    matcher.insert(buffer, pos);
                    length -= 1;
                }
            },
            _ => ()
        }

        pos += 1;
    }

    LZ77Encoded { data }

    //println!("Match lengths: {match_lengths:?}\nMatch offsets: {match_offsets:?}\nLiteral lengths: {literal_lengths:?}\nLiterals: {literals:?}");
}

pub fn lz77_decompress(encoded: LZ77Encoded) -> Vec<u8> {
    let mut decompressed = Vec::new();

    for data in encoded.data {
        match data {
            LZ77Data::Literal(val) => {
                decompressed.push(val);
            },
            LZ77Data::Match(length, offset) => {
                let mut start_pos = decompressed.len() - offset;
                for i in 0..length {
                    decompressed.push(decompressed[start_pos + (i % offset)]);
                }
            }
        }
    }

    decompressed
}

impl fmt::Display for LZ77Encoded{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut repr = String::new();

        for d in &self.data {
            match d{
                LZ77Data::Literal(val) => repr.push_str(format!("{} ", *val).as_str()),
                LZ77Data::Match(length, offset) => repr.push_str(format!("[Length: {} Offset: {}] ", *length, *offset).as_str())
            }
        }

        write!(f, "{repr}")
        
    }
}

#[cfg(test)]
mod tests {
    use crate::lz77::lz77_decompress;

    #[test]
    fn lz77_compress_decompress_simple() {
        use crate::lz77::{lz77_compress_simple};
        use std::{fs, time};
        
        let bytes = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
        //let bytes = "Blah blah blah blah blah!".as_bytes().to_vec();
        let start_time = time::Instant::now();
        let lz77_encoded = lz77_compress_simple(&bytes, 32 * 1024, 4);
        let elapsed_time = start_time.elapsed().as_millis();
        println!("Simple compression took {elapsed_time}ms at a speed of {}MB/s", ((bytes.len() as f32) / 1000000f32) / ((elapsed_time as f32) / 1000f32));

        let start_time = time::Instant::now();
        let lz77_decoded = lz77_decompress(lz77_encoded);
        let elapsed_time = start_time.elapsed().as_millis();
        println!("Decompression took {elapsed_time}ms at a speed of {}MB/s", ((lz77_decoded.len() as f32) / 1000000f32) / ((elapsed_time as f32) / 1000f32));

        assert!(lz77_decoded.len() == bytes.len(), "LZ77 compression and decompression resulted in different number of bytes");
        for i in 0..lz77_decoded.len() {
            assert!(lz77_decoded[i] == bytes[i], "LZ77 compression and decompression resulted in different bytes, at position {i}");
        }

    }
}