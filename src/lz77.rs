use std::collections::HashMap;
use std::fmt::{self};

const MAX_MATCH_NUM:usize = 1;

type LZ77MapKey = [u8; 3];
struct LZ77MatchFinder <'a>{
    buffer: &'a [u8],
    window_size:usize,
    min_match_len:usize,
    max_match_len:usize,
    head_map:HashMap<LZ77MapKey, usize>,
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

impl<'a> LZ77MatchFinder <'a> {
    fn new(buffer: &'a [u8], window_size:usize, min_match_len:usize, max_match_len:usize) -> Self {

        assert!(min_match_len > 0, "Minimum match length cannot be 0!");
        assert!(window_size > 0, "Window size must be greater than 1!");

        LZ77MatchFinder {
            buffer,
            window_size,
            min_match_len,
            max_match_len,
            head_map: HashMap::with_capacity(window_size),
            next_map: HashMap::with_capacity(window_size)
        }
    }

    // fn key_from_bytes(&self, pos: usize) -> LZ77MapKey{
    //     let mut hash:LZ77MapKey = 0;
    //     let byte_num = min(self.buffer.len() - pos, 3);
    
    //     for i in 0..byte_num{
    //         hash <<= 8;
    //         hash |= self.buffer[pos + i] as LZ77MapKey;
    //     }
    
    //     hash
    // }
    
    #[inline]
    fn key_from_bytes(&self, pos: usize) -> LZ77MapKey {
        let buf: &[u8] = &self.buffer[pos..(pos+3)];
        [buf[0], buf[1], buf[2]]
    }

    #[inline]
    fn insert(&mut self, pos: usize){
        let key = self.key_from_bytes(pos);

        if let Some(head) = self.head_map.get(&key){
            self.next_map.insert(pos, *head);
        }
        self.head_map.insert(key, pos);
    }

    fn find_match(&mut self, pos: usize) -> LZ77Data {
        let mut length:usize = 0;
        let mut offset:usize = 0;

        let min_pos:usize = if self.window_size > pos {0} else {pos - self.window_size};
        let mut next_option = self.head_map.get(&self.key_from_bytes(pos));
        let mut match_num = 0;
        
        while let Some(next) = next_option {
            let next = *next;
            if next < min_pos {break;}
            match_num += 1;
            if match_num > MAX_MATCH_NUM {break;}

            let match_len = self.match_len(pos + 3, next + 3) + 3;
            if match_len > length {
                length = match_len;
                offset = pos - next;
            }

            next_option = self.next_map.get(&next);
        }

        self.insert(pos);

        //println!("Pos: {pos} Best match: {best_match_pos} Best match length; {best_match_len}");

        if length >= self.min_match_len && offset >= self.min_match_len {LZ77Data::Match(length, offset)} else {LZ77Data::Literal(self.buffer[pos])}
    }

    fn find_matches(&mut self, pos: usize) -> Vec<LZ77Data> {
        let mut data = Vec::with_capacity(MAX_MATCH_NUM);

        let min_pos:usize = if self.window_size > pos {0} else {pos - self.window_size};
        let mut next_option = self.head_map.get(&self.key_from_bytes(pos));
        let mut match_num = 0;
        
        while let Some(next) = next_option {
            let next = *next;
            if next < min_pos {break;}

            match_num += 1;
            if match_num >= MAX_MATCH_NUM {break;}

            let length = self.match_len(pos + 3, next + 3) + 3;

            if length >= self.min_match_len {
                data.push(LZ77Data::Match(length, pos - next));
            }

            next_option = self.next_map.get(&next);
        }

        self.insert(pos);

        //println!("Pos: {pos} Best match: {best_match_pos} Best match length; {best_match_len}");

        data
    }

    #[inline]
    fn match_len(&self, source_pos: usize, match_pos: usize) -> usize {

        // let mut len:usize = 3;
        // let dist = source_pos - match_pos;

        // while len < self.max_match_len as usize && source_pos + len < self.buffer.len() - 3 && self.buffer[source_pos + len] == self.buffer[match_pos + (len % dist)] {
        //     len += 1;
        // }

        // len
        self.buffer[source_pos..]
            .iter()
            .take(self.max_match_len - 3)
            .zip(&self.buffer[match_pos..])
            .take_while(|&(a, b)| a == b)
            .count()
    }
}

pub fn lz77_compress_simple(buffer: &[u8], window_size: usize, min_match_len: usize, max_match_len: usize) -> LZ77Encoded{
    let mut matcher: LZ77MatchFinder = LZ77MatchFinder::new(buffer, window_size, min_match_len, max_match_len);
    let mut data = Vec::with_capacity(buffer.len());
    let mut pos = 0;

    while pos + 3 < buffer.len() {
        //println!("{pos} {} {}", buffer.len(), (pos as f32) / (buffer.len() as f32));

        let d = matcher.find_match(pos);
        data.push(d);

        match d {
            LZ77Data::Match(length, _) => {
                //println!("Found match of length {length} at distance {dist}, moving up to {}", pos + length);
                for pos_to_add in (pos..).take(length).skip(1) {
                    if pos_to_add + 3 <= buffer.len() {break;}
                    matcher.insert(pos_to_add);
                }
                pos += length;
            },
            _ => { pos += 1; }
        }
    }

    for byte in &buffer[pos..] {
        data.push(LZ77Data::Literal(*byte));
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
                let start_pos = decompressed.len() - offset;
                for i in 0..length {
                    decompressed.push(decompressed[start_pos + i]);
                }
            }
        }
    }

    decompressed
}

fn encoded_byte_num(encoded: &LZ77Encoded, match_size_bytes: usize) -> usize {
    let mut encoded_bytes = 0;

    for data in &encoded.data{
        encoded_bytes += match *data{
            LZ77Data::Literal(_) => 1,
            LZ77Data::Match(_, _) => match_size_bytes
        };
    }

    encoded_bytes
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
        use crate::lz77::{lz77_compress_simple, encoded_byte_num};
        use std::{fs, time};
        
        let bytes = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
        //let bytes = "Blah blah blah blah blah!".as_bytes().to_vec();
        let start_time = time::Instant::now();
        let lz77_encoded = lz77_compress_simple(&bytes, 0xFFFF, 3, 256);
        let encoded_num = encoded_byte_num(&lz77_encoded, 3);
        let elapsed_time = start_time.elapsed().as_millis();
        println!("Bytes unencoded:[{}] Bytes encoded:[{encoded_num}] Compression Ratio:[{}]\nTime:[{elapsed_time}]ms Speed:[{}]MB/s", bytes.len(), (encoded_num as f32) / (bytes.len() as f32), ((bytes.len() as f32) / 1000000f32) / ((elapsed_time as f32) / 1000f32));

        let start_time = time::Instant::now();
        let lz77_decoded = lz77_decompress(lz77_encoded);
        let elapsed_time = start_time.elapsed().as_millis();
        println!("Decompression time:[{elapsed_time}]ms Speed:[{}]MB/s", ((lz77_decoded.len() as f32) / 1000000f32) / ((elapsed_time as f32) / 1000f32));

        assert!(lz77_decoded.len() == bytes.len(), "LZ77 compression and decompression resulted in different number of bytes");
        for i in 0..lz77_decoded.len() {
            assert!(lz77_decoded[i] == bytes[i], "LZ77 compression and decompression resulted in different bytes at position {i}");
        }

    }
}