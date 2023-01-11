use crate::bitstream::{BitReader, BitWriter};
use crate::huffman::{HuffmanSymbol, HuffmanPath, compress_huffman, decompress_huffman};
use std::collections::HashMap;
use std::f32::MIN;
use std::hash::Hash;

type MatchLength = u32;
type MatchOffset = u32;

const MIN_CODE_LEN:usize = 9;
const MAX_CODE_LEN:usize = 12;
const MAX_CODE:u32 = (1 << 12);
const CLEAR_CODE:u32 = 256;
const EOD_CODE:u32 = 257;
const START_CODE:u32 = 258;

struct LZMatchFinder<'a> {
    bytes:&'a [u8],
    hash_map:Vec<usize>,
    hash_chain:Vec<usize>
}

fn fast_log2_u32(v:u32) -> u32 {
    if v == 0 {return 0;}

    31 - v.leading_zeros()
}

/// Creates a `HuffmanSymbol` given a `MatchLength`.
/// The symbol is just equal to the length if the length is less than
/// 16, and equal to 12 + log2(length) if greater. This allows for more compact
/// symbols, as long as you also denote the extra bits using 
/// `extra_bits_from_length`, since 
fn huffman_symbol_from_length(length: MatchLength) -> HuffmanSymbol {
    if length < 16 {
        return length as HuffmanSymbol;
    }

    (12 + fast_log2_u32(length)) as HuffmanSymbol
}

fn extra_bits_from_length(length: MatchLength) -> HuffmanSymbol {
    (length - (1 << fast_log2_u32(length))) as HuffmanSymbol
}

fn huffman_symbol_from_offset(offset: MatchOffset) -> HuffmanSymbol {
    if offset < 2 {return offset as HuffmanSymbol;}

    (1 + fast_log2_u32(offset)) as HuffmanSymbol
}

fn init_lzw_compression_table(map: &mut HashMap<Vec<u8>,u32>) {
    map.clear();
    for byte in 0..=255{
        map.insert(vec![byte], byte as u32);
    }
}
fn init_lzw_decompression_table(map: &mut HashMap<u32,Vec<u8>>) {
    map.clear();
    for byte in 0..=255{
        map.insert( byte as u32,vec![byte]);
    }
}

/// Very simple LZW compression.
/// 
/// I only made this to get a better understanding of how LZW encoding works.
/// 
/// In the event of a table overflow, the GIF approach of remaking the table is
/// used.
pub fn compress_lzw_simple(bytes: &[u8]) -> Vec<u8> {
    let mut writer = BitWriter::new();
    let mut code_len:usize = MIN_CODE_LEN;
    let mut curr_max_code:u32 = 1 << MIN_CODE_LEN;
    let mut map:HashMap<Vec<u8>,u32> = HashMap::with_capacity(1 << MAX_CODE_LEN);

    init_lzw_compression_table(&mut map);

    let mut code:u32 = START_CODE;
    let mut buffer = Vec::new();

    for byte in bytes{
        let mut buffer_new = buffer.clone();
        buffer_new.push(*byte);

        if map.contains_key(&buffer_new){

            buffer = buffer_new;

        } else{

            writer.write_bits_u32(*map.get(&buffer).unwrap(),code_len);
            map.insert(buffer_new, code);

            code += 1;
            if code >= MAX_CODE{
                //println!("Restarting table...");
                writer.write_bits_u32(CLEAR_CODE, code_len);

                code_len = MIN_CODE_LEN;
                curr_max_code = 1 << MIN_CODE_LEN;
                code = START_CODE;

            } else if code >= curr_max_code {
                code_len += 1;
                curr_max_code <<= 1;
                //println!("Increasing code length to {code_len}");
            }

            buffer = vec![*byte];
        }
    }
    if !buffer.is_empty(){
        writer.write_bits_u32(*map.get(&buffer).unwrap(),code_len);
    }

    writer.write_bits_u32(EOD_CODE, code_len);

    writer.get_bytes()
}

/// Very simple LZW compression.
/// 
/// I only made this to get a better understanding of how LZW encoding works.
/// 
/// In the event of a table overflow, the GIF approach of remaking the table is
/// used.
pub fn compress_lzw_simple_as_bytes(bytes: &[u8]) -> Vec<u32> {
    let mut codes = Vec::new();
    let mut code_len:usize = MIN_CODE_LEN;
    let mut curr_max_code:u32 = 1 << MIN_CODE_LEN;
    let mut map:HashMap<Vec<u8>,u32> = HashMap::with_capacity(1 << MAX_CODE_LEN);
    init_lzw_compression_table(&mut map);

    let mut code:u32 = START_CODE;
    let mut buffer = Vec::new();

    for byte in bytes{
        let mut buffer_new = buffer.clone();
        buffer_new.push(*byte);

        if map.contains_key(&buffer_new){
            buffer = buffer_new;
        } else{

            codes.push(*map.get(&buffer).unwrap());
            map.insert(buffer_new, code);

            code += 1;
            if code >= MAX_CODE{

                codes.push(CLEAR_CODE);

                init_lzw_compression_table(&mut map);

                code_len = MIN_CODE_LEN;
                curr_max_code = 1 << MIN_CODE_LEN;
                code = START_CODE;

            } else if code >= curr_max_code {
                code_len += 1;
                curr_max_code <<= 1;
            }

            //buffer = vec![*byte];
            buffer.clear();
            buffer.push(*byte);
        }
    }
    if !buffer.is_empty(){
        codes.push(*map.get(&buffer).unwrap());
        //writer.write_bits_u32(*map.get(&buffer).unwrap(),code_len);
    }

    codes.push(EOD_CODE);

    codes
}

/// Very stupid LZW decompression.
/// 
/// I only made this to get a better understanding of how LZW encoding works.
/// 
/// This uses the GIF method of starting over when the table gets too big.
pub fn decompress_lzw_simple(encoded_bytes: &[u8]) -> Vec<u8> {
    let mut reader = BitReader::new(encoded_bytes);
    let mut decoded_bytes = Vec::new();
    let mut code_len = MIN_CODE_LEN;
    let mut curr_max_code:u32 = 1 << MIN_CODE_LEN;

    let mut table:HashMap<u32, Vec<u8>> = HashMap::with_capacity(1 << MAX_CODE_LEN);
    init_lzw_decompression_table(&mut table);

    let mut prev = CLEAR_CODE;
    let mut code = START_CODE;
    let mut entry = Vec::new();

    loop {

        if prev == CLEAR_CODE {
            prev = reader.read_bits_into_u32(code_len).unwrap();
            entry = table.get(&prev).unwrap().clone();
            decoded_bytes.extend(&entry);
            continue;
        }

        //println!("Old entry: {entry:?}");
        match reader.read_bits_into_u32(code_len).unwrap() {
            CLEAR_CODE => {
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
                //println!("Code: {curr}");
                if table.contains_key(&curr){ //curr < code
                    //println!("In table");
                    entry = table.get(&curr).unwrap().clone();
                } else if curr == code {
                    //println!("Not in table");
                    entry = table.get(&prev).unwrap().clone();
                    entry.push(entry[0]);
                } else {
                    panic!("Bad compression with symbol {curr}, current code is {code}");
                }

                //println!("New entry: {entry:?}\n");
                decoded_bytes.extend(&entry);

                table.insert(code, [table.get(&prev).unwrap().clone(),vec![entry[0]]].concat());

                code += 1;
                if code >= curr_max_code {
                    curr_max_code <<= 1;
                    code_len += 1;
                    //println!("Increasing code length to {code_len}");
                }
                
                prev = curr;
            }
        }
        
    }

    decoded_bytes
}

pub fn decompress_lzw_simple_as_bytes(codes: &[u32]) -> Vec<u8> {
    let mut decoded_bytes = Vec::new();
    let mut code_len = MIN_CODE_LEN;
    let mut curr_max_code:u32 = 1 << MIN_CODE_LEN;

    let mut table:HashMap<u32, Vec<u8>> = HashMap::with_capacity(1 << MAX_CODE_LEN);
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


#[cfg(test)]
mod tests{
    #[test]
    pub fn fast_log2_u32_test() {
        for num in [97758u32, 93658u32, 86636u32, 2623u32, 75343u32, 46835u32, 88189u32, 52233u32, 3907u32, 64476u32, 12861u32, 46261u32, 60695u32, 38029u32, 61168u32, 77655u32, 97815u32, 49371u32, 49118u32, 29138u32, 16451u32, 1607u32, 10663u32, 93815u32, 75146u32, 42807u32, 68381u32, 1063u32, 4990u32, 63744u32, 10137u32, 43254u32, 21171u32, 83417u32, 33270u32, 40106u32, 64128u32, 53782u32, 85183u32, 44082u32, 25309u32, 30617u32, 21117u32, 38969u32, 17873u32, 65888u32, 36684u32, 92196u32, 87049u32, 6080u32, 32493u32, 58124u32, 48669u32, 99194u32, 85970u32, 12357u32, 91229u32, 6132u32, 97989u32, 84058u32, 37744u32, 4562u32, 59294u32, 65236u32, 16571u32, 56115u32, 73037u32, 35545u32, 41656u32, 42748u32, 31338u32, 77068u32, 44765u32, 15301u32, 96648u32, 65541u32, 54921u32, 17102u32, 96644u32, 94647u32, 79280u32, 1456u32, 87750u32, 56138u32, 37030u32, 93057u32, 97301u32, 96730u32, 82814u32, 41527u32, 56546u32, 53109u32, 18328u32, 38914u32, 55667u32, 75697u32, 35198u32, 17457u32, 42311u32, 97125u32] {
            assert!((num as f32).log2().floor() as u32 == crate::lzw::fast_log2_u32(num));
        }
    }

    #[test]
    pub fn lzw_simple_as_bytes_test() {
        use crate::lzw::{compress_lzw_simple_as_bytes, decompress_lzw_simple_as_bytes};
        use std::{fs, time};
        let bytes = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
        //let bytes = "TOBEORNOTTOBEORTOBEORNOT".as_bytes();
        let start_time = time::Instant::now();
        let encoded_bytes = compress_lzw_simple_as_bytes(&bytes);
        let elapsed_time = start_time.elapsed().as_millis();

        println!("Bytes unencoded: [{}] Bytes encoded:[{}] Compression ratio:[{}]\nTime:[{}]ms Speed:[{}]MB/s",bytes.len(), encoded_bytes.len(), (encoded_bytes.len() as f32) / (bytes.len() as f32), elapsed_time, ((bytes.len() as f32) / 1000f32) / (elapsed_time as f32));
        //println!("{encoded_bytes:?}");
        let start_time = time::Instant::now();
        let decoded_bytes = decompress_lzw_simple_as_bytes(&encoded_bytes);
        let elapsed_time = start_time.elapsed().as_millis();

        println!("Decompression time:[{}]ms Speed:[{}]MB/s", elapsed_time, ((encoded_bytes.len() as f32) / 1000f32) / (elapsed_time as f32));

        assert!(decoded_bytes.len() == bytes.len(), "Number of bytes changed during compression and decompression.");
        assert!(bytes.iter().zip(&decoded_bytes).all(|(a,b)| *a == *b), "Bytes compressed and decompressed incorrectly");
        
    }

    #[test]
    pub fn lzw_simple_test() {
        use crate::lzw::{compress_lzw_simple, decompress_lzw_simple};
        use std::{fs, time};
        let bytes = fs::read("lorem_ipsum").expect("File could not be opened and/or read");
        //let bytes = "TOBEORNOTTOBEORTOBEORNOT".as_bytes();
        let start_time = time::Instant::now();
        let encoded_bytes = compress_lzw_simple(&bytes);
        let elapsed_time = start_time.elapsed().as_millis();

        println!("Bytes unencoded: [{}] Bytes encoded:[{}] Compression ratio:[{}]\nTime:[{}]ms Speed:[{}]MB/s",bytes.len(), encoded_bytes.len(), (encoded_bytes.len() as f32) / (bytes.len() as f32), elapsed_time, ((bytes.len() as f32) / 1000f32) / (elapsed_time as f32));
        //println!("{encoded_bytes:?}");
        let start_time = time::Instant::now();
        let decoded_bytes = decompress_lzw_simple(&encoded_bytes);
        let elapsed_time = start_time.elapsed().as_millis();

        println!("Decompression time:[{}]ms Speed:[{}]MB/s", elapsed_time, ((encoded_bytes.len() as f32) / 1000f32) / (elapsed_time as f32));
        
        assert!(decoded_bytes.len() == bytes.len(), "Number of bytes changed during compression and decompression.");
        assert!(bytes.iter().zip(&decoded_bytes).all(|(a,b)| *a == *b), "Bytes compressed and decompressed incorrectly");
        
    }
}