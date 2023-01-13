use std::collections::{BinaryHeap};
use std::cmp::{Ordering, min, max};
use std::fmt::{self};
use crate::bitstream::{BitWriter, BitReader};

/// The maximum number of symbols that a Huffman table can use. `256` is
/// enough for one symbol per byte.
const MAX_SYMBOLS:usize = 256;
/// The number of bits needed to write the number of symbols.
const MAX_SYMBOLS_SIZE:usize = 8;
const CHUNK_SIZE_BITS:usize = 32;
const MAX_CODE_LEN:usize = 11;
const CODE_MASK:u32 = (1 << MAX_CODE_LEN) - 1;

pub type HuffmanSymbol = u8;
pub type HuffmanPath = u32;

/// A simple struct that `HuffmanTable`s use. Pretty much just a tuple.
/// 
/// Contains the symbol (a `u8`) and its level (a `i32`), the level being the 
/// symbol's depth in the Huffman tree.
#[derive(Debug, Eq, Clone, Copy)]
pub struct HuffmanTableData {
    pub symbol:HuffmanSymbol,
    pub level:i32
}

/// A simple struct for the data stored in a `HuffmanNode`.
/// 
/// Either is a leaf containing a symbol, or a node that points to a left and
/// right child node
#[derive(Debug)]
enum HuffmanNodeData {
    Node(Box<HuffmanNode>, Box<HuffmanNode>),
    Leaf(HuffmanSymbol)
}

/// A struct denoting a node in a Huffman tree.
/// 
/// The reason `freq` and `data` are separate is because all nodes have a
/// frequency, while some nodes are leaves and some aren't.
#[derive(Debug)]
struct HuffmanNode {
    freq: u64,
    data: HuffmanNodeData
}

/// The struct that can do Huffman encoding and decoding
/// 
/// The reason it's a struct is because there are instances where one may
/// want to encode only a bit at a time, rather than all at once.
pub struct HuffmanEncoder {
    writer: BitWriter,
    freq_table: Vec<u64>,
    max_symbols: usize,
    max_symbols_size: usize,
    table: HuffmanTable,
    code_map: HuffmanCodeMap,
    last_length: usize
}

pub struct HuffmanDecoder<'a>{
    reader: BitReader<'a>,
    decoded: Vec<u8>,
    table: HuffmanTable,
    symbol_map: Vec<HuffmanSymbol>,
    level_map: Vec<i32>
}

/// A `Vec` of `HuffmanTableData`. Its `len()` equals the number of symbols 
/// found.
type HuffmanTable = Vec<HuffmanTableData>;
/// A `Vec` of `(HuffmanPath, usize)`, which denote the path/code and length
/// of the code respectively. It's basically a fixed size Hash Map, as the
/// code for the `i`th symbol is found at the `i`th index, and the `len()` 
/// of the code map is equal to `MAX_SYMBOLS`. If the symbol has no code,
/// something that shouldn't happen in normal use, returns a `None`.
type HuffmanCodeMap = Vec<Option<(HuffmanPath,usize)>>;

impl PartialEq for HuffmanTableData {
    fn eq(&self, other: &HuffmanTableData) -> bool{
        self == other
    }
}

impl PartialOrd for HuffmanTableData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HuffmanTableData {
    fn cmp(&self, other: &Self) -> Ordering{
        self.level.cmp(&other.level)
    }
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &HuffmanNode) -> bool{
        self.freq == other.freq
    }
}

impl Eq for HuffmanNode {}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering{
        other.freq.cmp(&self.freq)
    }
}

impl fmt::Display for HuffmanNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        // match &self.data {
        //     HuffmanNodeData::Node(left, right) => write!(f, "Frequency:[{}] Left:[{}] Right[{}]", self.freq, *left, *right),
        //     HuffmanNodeData::Leaf(symbol) => write!(f, "Frequency:[{}] Symbol:[{:x}]", self.freq, symbol)
        // }
        match &self.data {
            HuffmanNodeData::Node(left, right) => write!(f,"Left:[\n{}\n]\nRight:[\n{}\n]",*left,*right),
            HuffmanNodeData::Leaf(symbol) => write!(f, "Frequency:[{}] Symbol:[{:x}]", self.freq, symbol)
        }
        
    }
}

impl HuffmanNode {
    /// Creates a leaf `HuffmanNode`.
    pub fn leaf(symbol: u8, freq: u64) -> Self{
        HuffmanNode{
            freq,
            data:HuffmanNodeData::Leaf(symbol)
        }
    }

    /// Creates a regular `HuffmanNode`. Assumes that the left and right
    /// child nodes have already been created.
    pub fn node(left: HuffmanNode, right: HuffmanNode) -> Self{
        HuffmanNode {
            freq:left.freq + right.freq,
            data:HuffmanNodeData::Node(Box::new(left), Box::new(right))
        }
    }

    fn leaves_helper(&self, leaves: &mut HuffmanTable, level: i32){
        match &self.data{
            HuffmanNodeData::Node(left, right) => {
                left.leaves_helper(leaves, level+1);
                right.leaves_helper(leaves, level+1);
            },
            HuffmanNodeData::Leaf(symbol) => {
                leaves.push(HuffmanTableData { symbol: *symbol, level: level });
            }
        }
    }

    /// Parses the Huffman tree using DFS, creating a `HuffmanTable`. The 
    /// `HuffmanTable` is sorted from lowest to highest level.
    pub fn leaves(&self, leaves: &mut HuffmanTable){
        //let mut leaves = Vec::with_capacity(max_symbols);
        match &self.data{
            HuffmanNodeData::Leaf(symbol) => {
                leaves.push(HuffmanTableData { symbol: *symbol, level: 1 });
            },
            HuffmanNodeData::Node(_, _) => {
                self.leaves_helper(leaves, 0);
            }
        }
    }

}

impl HuffmanEncoder {
    pub fn new(bytes: &[u8], max_symbols: usize) -> Self{

        let max_symbols =  min(max_symbols, MAX_SYMBOLS);

        let mut encoder = HuffmanEncoder {
            writer:BitWriter::new(),
            freq_table:Vec::with_capacity(max_symbols),
            max_symbols:max_symbols,
            max_symbols_size:((max_symbols as f32).log2().ceil() as usize),
            table:Vec::with_capacity(max_symbols),
            code_map:vec![None; max_symbols],
            last_length:0
        };
        encoder.freq_table.resize(max_symbols, 0);

        encoder
    }

    /// Builds a frequency table given a slice of bytes.
    /// 
    /// `build_frequency_table(&bytes)[i]` denotes the number of times the symbol
    /// `i` appears in `bytes`.
    pub fn build_frequency_table(&mut self, bytes: &[u8]) {
        self.freq_table.fill(0);

        for byte in bytes{
            self.freq_table[*byte as usize] += 1;
        }
    }

    pub fn scan_byte(&mut self, byte: u8) {
        self.freq_table[byte as usize] += 1;
    }

    /// Builds a huffman table.
    /// 
    /// Creates a frequency table using `build_frequency_table()`, builds a Huffman
    /// tree out of `HuffmanNode`s using the frequency table with a `BinaryHeap`, 
    /// then turns that huffman tree into a `HuffmanTable`.
    pub fn build_huffman_table(&mut self) {
        let mut node_heap:BinaryHeap<HuffmanNode> = BinaryHeap::new();
        let mut symbol_num = 0;
        for byte in 0..self.max_symbols {
            if self.freq_table[byte] > 0{
                node_heap.push(HuffmanNode::leaf(byte as u8, self.freq_table[byte]));
                symbol_num += 1;
            }
        }

        while node_heap.len() > 1{
            let left = node_heap.pop().unwrap();
            let right = node_heap.pop().unwrap();
            node_heap.push(HuffmanNode::node(left, right));
        }

        self.table.clear();
        node_heap.pop().unwrap().leaves(&mut self.table);
        self.table.sort();

        self.limit_huffman_table_code_sizes();
        self.write_huffman_table();
        self.build_huffman_code_map();
    }

    /// Limits the maximum levels of the symbols in the `HuffmanTable`, increasing
    /// and decreasing the levels of symbols accordingly.
    /// 
    /// This results in some symbols having longer codes, but it makes decompression
    /// much faster, as it gives a definite upper bound on the size of paths.
    /// 
    /// I'm not sure what happens when the `max_code_length` is too small, so just
    /// in case it panics if the `max_code_length` isn't enough to store all the
    /// symbols in the `HuffmanTable`
    fn limit_huffman_table_code_sizes(&mut self){

        assert!((self.table.len() as f32).log2().ceil() as usize <= MAX_CODE_LEN, "Maximum code length of [{}] not large enough to store all [{}] symbols, needs length of at least [{}]", MAX_CODE_LEN, self.table.len(), (self.table.len() as f32).log2().ceil() as i32);

        let mut k = 0;
        let k_max:usize = (1 << MAX_CODE_LEN) - 1;

        for i in (0..self.table.len()).rev(){
            self.table[i].level = min(self.table[i].level, MAX_CODE_LEN as i32);
            k += 1 << ((MAX_CODE_LEN as i32) - self.table[i].level);
        }

        for i in (0..self.table.len()).rev(){
            if k <= k_max {
                break;
            }
            while self.table[i].level < (MAX_CODE_LEN as i32) {
                self.table[i].level += 1;
                k -= 1 << ((MAX_CODE_LEN as i32) - self.table[i].level);
            }
        }
        for i in 0..self.table.len(){
            while k + (1 << ((MAX_CODE_LEN as i32) - self.table[i].level)) <= k_max {
                k += 1 << ((MAX_CODE_LEN as i32) - self.table[i].level);
                self.table[i].level -= 1;
            }
        }
    }

    /// Writes a `HuffmanTable` to a given `BitWriter`.
    /// 
    /// First writes `MAX_SYMBOLS_SIZE` bits denoting the number of symbols in
    /// the `HuffmanTable` (`huffman_table.len()`) and `MAX_SYMBOLS_SIZE` bits
    /// denoting the number of bits used to encode a level 
    /// (`bits_per_level`). If there's only one symbol, write `1` instead.
    /// 
    /// For each symbol in the `HuffmanTable`, write `MAX_SYMBOLS_SIZE` bits
    /// denoting the symbol itself, and `bits_per_level` bits denoting the level
    /// of the symbol. This is better than writing the code itself, since the codes
    /// can get quite long.
    fn write_huffman_table(&mut self) {

        assert!(self.table.len() <= MAX_SYMBOLS, "The given Huffman table has too many symbols");

        self.writer.write_bits_u32(self.table.len() as u32, MAX_SYMBOLS_SIZE);

        let max_level = self.table.iter().max().unwrap().level; //Is this really necessary? I guess every little bit helps...
        self.writer.write_bits_u32(max_level as u32, MAX_SYMBOLS_SIZE);
        let bits_per_level = max((max_level as f32).log2().ceil() as usize, 1);
        //println!("{max_level} {bits_per_level}");

        for data in &self.table{
            let symbol = data.symbol;
            let level = data.level;
            self.writer.write_bits_u32(symbol as u32, MAX_SYMBOLS_SIZE);
            self.writer.write_bits_u32(level as u32, bits_per_level);
        }
    }

    /// Prints the encoder's `HuffmanTable`
    pub fn print_huffman_table(&self) {
        for data in &self.table{
            println!("Symbol: [{:x}] Level: [{}]", data.symbol, data.level);
        }
    }

    /// Builds a `HuffmanCodeMap` from a given `HuffmanTable`
    /// 
    /// Relies on the fact that all the nodes on a specific level can have codes
    /// just by incrementing the code of the leaf next to it on the same level, if
    /// that makes any sense.
    fn build_huffman_code_map(&mut self) {
        self.code_map.fill(None);

        let mut code:HuffmanPath = 0;
        let mut last_level:i32 = -1;

        for data in &self.table{
            let symbol = data.symbol;
            let level = data.level;

            if last_level != level{
                if last_level != -1 {
                    code += 1;
                    code <<= level - last_level;
                }
                last_level = level;
            } else {
                code += 1;
            }

            self.code_map[symbol as usize] = Some((code, level as usize));
        }

    }

    /// Prints the encoder's Huffman code map
    pub fn print_huffman_code_map(&self) {
        for symbol in 0..self.max_symbols{
            if let Some((code, level)) = &self.code_map[symbol]{
                print!("Symbol: [{:x}] Code:[", symbol);
                for i in (0..*level).rev() {
                    print!("{}", if (*code & (1 << i)) > 0 {1} else {0});
                }
                println!("]");
            }
        }
    }

    /// Encodes a single given symbol. `panic`s if the symbol isn't in the code
    /// map, which should never happen.
    /// 
    /// WARNING: After encoding symbols, remember to `finish` the encoder to 
    /// add the proper padding!
    pub fn encode_symbol(&mut self, symbol:HuffmanSymbol) {
        if let Some((code, length)) = self.code_map[symbol as usize]{
            self.writer.write_bits_u32(code, length);
            self.last_length = length;
        } else {
            panic!("Encoded symbol not found, this should never happen...");
        }
    }

    /// Encodes a slice of symbols.
    /// 
    /// WARNING: After encoding symbols, remember to `finish` the encoder to
    /// add the proper padding!
    pub fn encode_symbols(&mut self, symbols: &[HuffmanSymbol]) {
        self.writer_mut().write_bits_u32(symbols.len() as u32, CHUNK_SIZE_BITS);

        for symbol in symbols {
            if let Some((code, length)) = self.code_map[*symbol as usize]{
                self.writer.write_bits_u32(code, length);
                //self.last_length = length;
            }
        }
       self.last_length = self.code_map[symbols[symbols.len() - 1] as usize].unwrap().1;
    }

    /// Finish the encoder, adding padding to the end to make sure that the
    /// decoder works.
    pub fn finish(&mut self) {
        //println!("{} {}", self.last_length, self.max_code_length - self.la);
        self.writer.write_bits_u32(0, MAX_CODE_LEN - self.last_length);
    }

    /// Returns a reference to the encoder's `BitWriter`
    pub fn writer(&self) -> &BitWriter{
        &self.writer
    }

    /// Returns a mutable reference to the encoder's `BitWriter`
    pub fn writer_mut(&mut self) -> &mut BitWriter{
        &mut self.writer
    }

    /// Get the encoded bytes of the Huffman encoder's writer. Do this
    /// after `finish`ing the encoder to make sure padding is present.
    pub fn encoded_bytes(&self) -> Vec<u8> {
        self.writer().get_bytes()
    }


}

impl<'a> HuffmanDecoder<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        let mut reader = BitReader::new(bytes);
        HuffmanDecoder { 
            reader, 
            decoded: Vec::new(),
            table: HuffmanTable::with_capacity(MAX_SYMBOLS), 
            symbol_map: vec![0; 1 << MAX_CODE_LEN], 
            level_map: vec![0; 1 << MAX_CODE_LEN], 
        }
    }

    pub fn read_huffman_table(&mut self) {
        let symbol_num = self.reader.read_bits_into_u32(MAX_SYMBOLS_SIZE).unwrap() as usize;
        let max_level = self.reader.read_bits_into_u32(8).unwrap() as i32;
        let bits_per_level = max((max_level as f32).log2().ceil() as usize,1);
        //println!("Preliminary data read");
        //println!("Chunk size: [{chunk_size}] Symbol num: [{symbol_num}] Max level: [{max_level}] Bits per level: [{bits_per_level}]");

        self.table.clear();
        for _ in 0..symbol_num{
            let symbol = self.reader.read_bits_into_u8(MAX_SYMBOLS_SIZE).unwrap() as u8;
            let level = self.reader.read_bits_into_u32(bits_per_level).unwrap() as i32;
            self.table.push(HuffmanTableData{ symbol, level });
        }
        //println!("Huffman table read");
        //print_huffman_table(&huffman_table);

        self.fill_huffman_symbol_and_level_maps();

    }

    /// Fills a symbol and level maps.
    /// 
    /// It's basically the same as `build_huffman_code_map`, except instead
    /// of building a `HuffmanCodeMap` we're instead filling in two slices
    /// `symbol_table` and `level_table`. `symbol_table[i]` denotes the symbol 
    /// reached using code `i`, and `level_table[i]` denotes the actual length 
    /// of the code `i`. 
    /// 
    /// If, say, `000` is a path, if the maximum path length is `8`, we can be sure that
    /// the paths `0b00000000..0b00011111` all lead to the same symbol. Furthermore,
    /// this allows us to read the maximum path length of bis from the buffer, 
    /// making decompression much easier. This is why limiting the maximum path 
    /// length is so important.
    fn fill_huffman_symbol_and_level_maps(&mut self) {
        //let mut map:HuffmanSymbolMap = vec![HuffmanTableData { symbol:0, level:0 }; 1 << max_level];

        let mut code:HuffmanPath = 0;
        let mut last_level = -1;

        for data in &self.table{
            let symbol = data.symbol;
            let level = data.level;

            if last_level != level{
                if last_level != -1 {
                    code += 1;
                    code <<= level - last_level;
                }
                last_level = level;
            } else {
                code += 1;
            }

            //let reversed_code = reverse_u32(code);
            let start_code = (code << (MAX_CODE_LEN as i32 - level)) as usize;
            let end_code = (start_code | ((1 << (MAX_CODE_LEN as i32 - level))-1)) as usize;
            //println!("{} {level} {code:b} {start_code:064b} {end_code:064b}", self.max_code_length);
            self.symbol_map[start_code..=end_code].fill(symbol);
            self.level_map[start_code..=end_code].fill(level);
            // for i in 0..(1 << (max_level - level)){
            //     //println!("{:011b}",(code | (i << level)) as usize);
            //     map[i | (code << (max_level - level)) as usize] = HuffmanTableData { symbol, level };
            // }
            //map[code as usize] = Some(HuffmanTableData { symbol, level });
        }

    }

    fn decode(&mut self) {
        let chunk_size = self.reader.read_bits_into_u32(CHUNK_SIZE_BITS).unwrap() as usize;
        //println!("Symbol map generated");
        //print_huffman_symbol_map(&symbol_map);

        let mut bytes_to_decode:usize = chunk_size;
        let mut path:u32 = 0;
        let mut bits_to_read = MAX_CODE_LEN as i32;

        while bytes_to_decode > 0{
            //println!("{:011b} {}",path, bits_to_read);
            //let bit = bitstream.read_bit().unwrap();
            path |= self.reader.read_bits_into_u32(bits_to_read as usize).unwrap();
            //println!("{:011b}",path);
            //let data = symbol_map[path as usize];
            let symbol = self.symbol_map[path as usize];
            let level = self.level_map[path as usize];

            self.decoded.push(symbol);
            path <<= level;
            path &= CODE_MASK;
            bytes_to_decode -= 1;
            bits_to_read = level;
        }


        // println!("Chunk decoded");

        // while bytes_to_decode > 0{
        //     //println!("{:011b} {}",path, bits_to_read);
        //     //let bit = bitstream.read_bit().unwrap();
        //     let path = self.reader.peek_bits_into_u32(self.max_code_length).unwrap();
        //     //println!("{:011b}",path);
        //     //let data = symbol_map[path as usize];
        //     let symbol = self.symbol_map[path as usize];
        //     let level = self.level_map[path as usize];

        //     self.decoded.push(symbol);
        //     self.reader.read_bits_into_u32(level as usize);
        //     //path <<= level;
        //     //path &= mask;
        //     bytes_to_decode -= 1;
        // }
    }

    pub fn decoded_bytes(&self) -> Vec<u8> {
        self.decoded.clone()
    }

    pub fn reader(&self) -> &BitReader {
        &self.reader
    }

    pub fn reader_mut(&mut self) -> &'a mut BitReader {
        &mut self.reader
    }

}

/// Encodes a slice of bytes using Huffman encoding.
/// 
/// This encoding uses chunking, which can result in better compression.
/// `chunk_size` denotes the size of each chunk. If you don't want any
/// chunking, set `chunk_size` to `usize::MAX`. Otherwise, I've found
/// that `1 << 18`, or roughly 256 KB, is a good size for chunks.
/// 
/// `max_path_size` denotes the maximum length of the Huffman paths of each
/// symbol. This is necessary for making decompression fast. Note that using
/// larger `max_path_size`s results in decompression taking up much more space.
/// Therefore, it's advised to make `max_path_size` as small as possible.
/// If you're unsure what to set this to, I've found that `11` is a good length.
pub fn compress_huffman(bytes: &[u8], chunk_size:usize) -> Vec<u8> {

    let mut encoder = HuffmanEncoder::new(bytes, MAX_SYMBOLS);

    //encoder.writer_mut().write_bits_u32(max_path_size as u32, 8);
    let chunk_size = min(chunk_size, (1 << 24) - 1);
    for i in (0..bytes.len()).step_by(chunk_size){
        let chunk = &bytes[i..min(bytes.len(),i+chunk_size)];
        
        encoder.build_frequency_table(chunk);
        encoder.build_huffman_table();

        //write_huffman_table(&mut bitstream, &huffman_table);
        //println!("{}",chunk_size, bitstream);
        //println!("Huffman table written");
        encoder.encode_symbols(chunk);
        // for byte in chunk{
        //     encoder.encode_symbol(*byte);
        // }
        encoder.finish();
        //println!("Number of symbols: [{}] Smallest code length: [{}] Largest code length: [{}]", huffman_table.len(),  huffman_table.iter().min().unwrap().level,  huffman_table.iter().max().unwrap().level);
    }
    println!("File encoded");

    encoder.encoded_bytes()

}

///Decodes a slice of bytes encoded using Huffman encoding.
/// 
/// WARNING: I don't know what this does if the encoded bytes weren't created
/// using my `compress_huffman` function. Therefore, I'd advise you don't
/// use it on anything not created using this function.
pub fn decompress_huffman(encoded_bytes: &[u8]) -> Vec<u8> {
    let mut decoder = HuffmanDecoder::new(encoded_bytes);

    while decoder.reader().remaining_bits() > CHUNK_SIZE_BITS {
        //println!("{}",decoder.reader().remaining_bits());
        
        decoder.read_huffman_table();
        decoder.decode();
        //println!("Huffman table read");
        //print_huffman_table(&huffman_table);

        //fill_huffman_symbol_maps(&huffman_table, &mut symbol_table, &mut level_table, total_max_level);

    }

    decoder.decoded_bytes()
}

#[cfg(test)]
mod tests{

    fn huffman_test(chunk_size: usize){
        use std::{fs, time};
        use crate::huffman::{compress_huffman, decompress_huffman};
        let contents = fs::read("lorem_ipsum").expect("File could not be opened and/or read");

        let start_time = time::Instant::now();
        let encoded_bytes = compress_huffman(&contents, chunk_size);
        let elapsed_time = start_time.elapsed().as_millis();

        println!("Bytes unencoded: [{}] Bytes encoded:[{}] Compression ratio:[{}]\nTime:[{}]ms Speed:[{}]MB/s",contents.len(), encoded_bytes.len(), (encoded_bytes.len() as f32) / (contents.len() as f32), elapsed_time, ((contents.len() as f32) / 1000f32) / (elapsed_time as f32));
        
        let start_time = time::Instant::now();
        let decoded_bytes = decompress_huffman(&encoded_bytes);
        let elapsed_time = start_time.elapsed().as_millis();

        println!("Decompression time:[{}]ms Speed:[{}]MB/s", elapsed_time, ((encoded_bytes.len() as f32) / 1000f32) / (elapsed_time as f32));
        
        assert!(contents.len() == decoded_bytes.len(), "Number of bytes different after encoding and decoding");
        for i in 0..contents.len(){
            assert!(contents[i] == decoded_bytes[i], "Byte at position {i} different after encoding and decoding [{}] -> [{}]", contents[i], decoded_bytes[i]);
        }
    }

    #[test]
    pub fn huffman_test_basic(){
        huffman_test(usize::MAX);
    }

    #[test]
    pub fn huffman_test_chunking(){
        huffman_test(1<<14);
    }

}