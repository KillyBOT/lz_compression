use crate::bitstream::{BitReader, BitWriter};
use crate::huffman::{HuffmanSymbol, HuffmanPath, compress_huffman, decompress_huffman};
use std::char::MAX;
use std::collections::HashMap;

const MIN_CODE_LEN:usize = 9;
const MAX_CODE_LEN:usize = 12;
const MAX_CODE:u16 = 1 << MAX_CODE_LEN;
const START_MAX_CODE:u16 = 1 << MIN_CODE_LEN;
const CLEAR_CODE:u16 = 256;
const EOD_CODE:u16 = 257;
const START_CODE:u16 = 258;

#[derive(Clone, Copy)]
struct LZWEDecompressionTableData {
    prev: u16,
    next: u16,
    byte: u8
}

impl LZWEDecompressionTableData {
    fn new() -> Self {
        LZWEDecompressionTableData { prev:0, next:0, byte: 0}
    }
}

fn new_lzw_decompression_table()-> Vec<LZWEDecompressionTableData>{
    let mut table = vec![LZWEDecompressionTableData::new(); MAX_CODE as usize];
    for i in 0..=255{
        table[i as usize].byte = i;
    }

    table
}

/// LZW compression.
/// 
/// In the event of a table overflow, the GIF approach of remaking the table is
/// used.
/// 
/// This implementation is based on the C implementation found at
/// https://rosettacode.org/wiki/LZW_compression#C. I think this implementation
/// is what GIF uses, but I'm not sure.
pub fn compress_lzw(bytes: &[u8]) -> Vec<u8> {
    let mut writer = BitWriter::new();
    let mut code_len:usize = MIN_CODE_LEN;
    let mut curr_max_code:u16 = START_MAX_CODE;
    let mut table:HashMap<(u16, u16), u16> = HashMap::with_capacity(MAX_CODE as usize);

    let mut code = bytes[0] as u16;
    let mut next_code = START_CODE;
    
    for byte in &bytes[1..] {
        let byte = *byte as u16;
        
        //let next_option = table[code as usize].next[byte as usize];

        if let Some(next) = table.get(&(code, byte)){
            code = *next;
        } else {
            //println!("{code}");
            writer.write_bits_u16(code, code_len);
            table.insert((code, byte), next_code);
            code = byte;

            next_code += 1;

            if next_code == curr_max_code {
                code_len += 1;
                curr_max_code <<= 1;
                //println!("Increasing code length to {code_len}");
                if code_len == MAX_CODE_LEN {
                    writer.write_bits_u16(CLEAR_CODE, code_len);
                    
                    code_len = MIN_CODE_LEN;
                    curr_max_code = START_MAX_CODE;
                    next_code = START_CODE;

                    table.clear();
                }
            }
        }
    }

    writer.write_bits_u16(code,code_len);
    writer.write_bits_u16(EOD_CODE, code_len);

    writer.get_bytes()
}

/*
/// Simple LZW compression, 
/// 
/// I only made this to get a better understanding of how LZW encoding works.
/// 
/// In the event of a table overflow, the GIF approach of remaking the table is
/// used.
pub fn compress_lzw_as_bytes(bytes: &[u8]) -> Vec<u16> {
    let mut codes = Vec::new();
    let mut map:HashMap<Vec<u8>,u16> = HashMap::with_capacity(1 << MAX_CODE_LEN);
    init_lzw_compression_table(&mut map);

    let mut code:u16 = START_CODE;
    let mut buffer = Vec::new();

    for byte in bytes{
        let mut buffer_new = buffer.clone();
        buffer_new.push(*byte);

        if map.contains_key(&buffer_new){
            buffer = buffer_new;
        } else{
            codes.push(*map.get(&buffer).unwrap());
            map.insert(buffer_new, code);

            buffer.clear();
            buffer.push(*byte);

            code += 1;
            if code >= MAX_CODE{

                codes.push(CLEAR_CODE);

                init_lzw_compression_table(&mut map);

                code = START_CODE;

            }

        }
    }
    if !buffer.is_empty(){
        codes.push(*map.get(&buffer).unwrap());
    }

    codes.push(EOD_CODE);

    codes
}
*/

/// LZW decompression.
/// 
/// In the event of a table overflow, the GIF approach of remaking the table is
/// used.
/// 
/// This implementation is based on the C implementation found at
/// https://rosettacode.org/wiki/LZW_compression#C. I think this implementation
/// is what GIF uses, but I'm not sure.
pub fn decompress_lzw(encoded_bytes: &[u8]) -> Vec<u8> {
    let mut reader = BitReader::new(encoded_bytes);
    let mut decoded_bytes = Vec::new();

    let mut code_len = MIN_CODE_LEN;
    let mut curr_max_code:u16 = START_MAX_CODE;

    let mut table = new_lzw_decompression_table();

    let mut next_code = START_CODE;

    loop {
        //Read a code from the bit reader. This should never panic.
        let code = reader.read_bits_into_u16(code_len).unwrap();
        
        //If the EOD code is read, you reached the end of the encoded data, so exit
        if code == EOD_CODE { 
            break; 
        }
        //If the CLEAR_CODE code is read, restart the table
        if code == CLEAR_CODE {
            table = new_lzw_decompression_table();
            code_len = MIN_CODE_LEN;
            curr_max_code = START_MAX_CODE;
            next_code = START_CODE;
            continue;
        }

        //The read code should never be larger than the next code
        if code >= next_code {
            panic!("Bad compression with symbol {code}");
        }

        let mut curr = code;
        table[next_code as usize].prev = code;

        //While the current code isn't a byte
        while curr > u8::MAX as u16 {
            let tmp = table[curr as usize].prev;
            table[tmp as usize].next = curr;
            curr = tmp;
        }

        table[(next_code as usize) - 1].byte = curr as u8;

        while table[curr as usize].next > 0{
            decoded_bytes.push(table[curr as usize].byte);
            let tmp = table[curr as usize].next;
            table[curr as usize].next = 0;
            curr = tmp;
        }
        decoded_bytes.push(table[curr as usize].byte);

        next_code += 1;
        if next_code >= curr_max_code {
            code_len += 1;
            curr_max_code <<= 1;
        }



    }

    decoded_bytes
}

/*
pub fn decompress_lzw_as_bytes(codes: &[u16]) -> Vec<u8> {
    let mut decoded_bytes = Vec::new();
    let mut code_len = MIN_CODE_LEN;
    let mut curr_max_code:u16 = START_MAX_CODE;

    let mut table:HashMap<u16, Vec<u8>> = HashMap::with_capacity(1 << MAX_CODE_LEN);
    init_lzw_decompression_table(&mut table);

    let mut prev = CLEAR_CODE;
    let mut code = START_CODE;
    //let mut entry = Vec::new();
    let mut entry = Vec::new();

    //decoded_bytes.extend(&entry);

    for i in codes {
        //println!("Old entry: {entry:?}");
        //println!("{i}");

        if prev == CLEAR_CODE{
            prev = *i;
            entry = table.get(&prev).unwrap().clone();
            decoded_bytes.extend(&entry);
            continue;
        }
        
        match *i {
            CLEAR_CODE  => {
                //println!("Clear code found, resetting table...");
                init_lzw_decompression_table(&mut table);

                prev = CLEAR_CODE;
                code = START_CODE;
                code_len = MIN_CODE_LEN;
                curr_max_code = 1 << MIN_CODE_LEN;

            },
            EOD_CODE => {
                //println!("EOD code found");
                break;
            },
            curr => {
                if table.contains_key(&curr){ //curr < code
                    //println!("In table");
                    entry = table.get(&curr).unwrap().clone();
                } else if curr == code {
                    //println!("Not in table");
                    entry = table.get(&prev).unwrap().clone();
                    entry.push(entry[0]);
                } 
                else {
                    panic!("Bad compression with symbol {curr}, current code is {code} and max code is {curr_max_code}");
                }
                //println!("New entry: {entry:?}\n");
                decoded_bytes.extend(&entry);

                table.insert(code, [table.get(&prev).unwrap().clone(),vec![entry[0]]].concat());

                code += 1;
                if code == curr_max_code {
                    curr_max_code <<= 1;
                    code_len += 1;
                }
                
                prev = curr;
            }
        }
        
    }

    decoded_bytes
}
*/


#[cfg(test)]
mod tests{

    /*
    #[test]
    pub fn lzw_as_bytes_test() {
        use crate::lzw::{compress_lzw_as_bytes, decompress_lzw_as_bytes};
        use std::{fs, time};
        let bytes = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
        //let bytes = "TOBEORNOTTOBEORTOBEORNOT".as_bytes();
        let start_time = time::Instant::now();
        let encoded_bytes = compress_lzw_as_bytes(&bytes);
        let elapsed_time = start_time.elapsed().as_millis();

        println!("Bytes unencoded: [{}] Bytes encoded:[{}] Compression ratio:[{}]\nTime:[{}]ms Speed:[{}]MB/s",bytes.len(), encoded_bytes.len(), (encoded_bytes.len() as f32) / (bytes.len() as f32), elapsed_time, ((bytes.len() as f32) / 1000f32) / (elapsed_time as f32));
        let start_time = time::Instant::now();
        let decoded_bytes = decompress_lzw_as_bytes(&encoded_bytes);
        let elapsed_time = start_time.elapsed().as_millis();

        println!("Decompression time:[{}]ms Speed:[{}]MB/s", elapsed_time, ((encoded_bytes.len() as f32) / 1000f32) / (elapsed_time as f32));

        assert!(decoded_bytes.len() == bytes.len(), "Number of bytes changed during compression and decompression.");
        assert!(bytes.iter().zip(&decoded_bytes).all(|(a,b)| *a == *b), "Bytes compressed and decompressed incorrectly");
        
    }
    */

    #[test]
    pub fn lzw_test() {
        use crate::lzw::{compress_lzw, decompress_lzw};
        use std::{fs, time};
        use rand::prelude::*;
        
        let bytes = fs::read("enwik8").expect("File could not be opened and/or read");
        //let bytes = "TOBEORNOTTOBEORTOBEORNOT".as_bytes();
        // let byte_num = 4096;
        // let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2123);
        // let mut bytes = Vec::with_capacity(byte_num);
        // for _ in 0..byte_num {bytes.push(rng.gen::<u8>());}

        let start_time = time::Instant::now();
        let encoded_bytes = compress_lzw(&bytes);
        let elapsed_time = start_time.elapsed().as_millis();

        println!("Bytes unencoded: [{}] Bytes encoded:[{}] Compression ratio:[{}]\nTime:[{}]ms Speed:[{}]MB/s",bytes.len(), encoded_bytes.len(), (encoded_bytes.len() as f32) / (bytes.len() as f32), elapsed_time, ((bytes.len() as f32) / 1000f32) / (elapsed_time as f32));
        //println!("{encoded_bytes:?}");
        let start_time = time::Instant::now();
        let decoded_bytes = decompress_lzw(&encoded_bytes);
        let elapsed_time = start_time.elapsed().as_millis();

        println!("Decompression time:[{}]ms Speed:[{}]MB/s", elapsed_time, ((encoded_bytes.len() as f32) / 1000f32) / (elapsed_time as f32));
        
        assert!(decoded_bytes.len() == bytes.len(), "Number of bytes changed during compression and decompression.");
        assert!(bytes.iter().zip(&decoded_bytes).all(|(a,b)| *a == *b), "Bytes compressed and decompressed incorrectly");
    }
}