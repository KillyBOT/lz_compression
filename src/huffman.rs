use std::collections::{BinaryHeap};
use std::cmp::{Ordering, min, max};
use std::fmt::{self};
use crate::bitstream::{BitWriter, BitReader};

/// The maximum number of symbols that a Huffman table can use. `256` is
/// enough for one symbol per byte.
const MAX_SYMBOLS:usize = 256;
/// 
const MAX_SYMBOLS_SIZE:usize = 8;

pub type HuffmanSymbol = u8;
type HuffmanPath = u32;

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
    pub fn leaves(&self) -> HuffmanTable{
        let mut leaves = Vec::with_capacity(MAX_SYMBOLS);
        match &self.data{
            HuffmanNodeData::Leaf(symbol) => {
                leaves.push(HuffmanTableData { symbol: *symbol, level: 1 });
            },
            HuffmanNodeData::Node(_, _) => {
                self.leaves_helper(&mut leaves, 0);
                leaves.sort();
            }
        }

        leaves
    }

}

/// Builds a frequency table given a slice of bytes.
/// 
/// `build_frequency_table(&bytes)[i]` denotes the number of times the symbol
/// `i` appears in `bytes`.
fn build_frequency_table(bytes: &[u8]) -> Vec<u64> {
    let mut freq_vec = Vec::with_capacity(MAX_SYMBOLS);
    freq_vec.resize(MAX_SYMBOLS, 0);

    for byte in bytes{
        freq_vec[*byte as usize] += 1;
    }

    freq_vec
}

/// Builds a huffman table.
/// 
/// Creates a frequency table using `build_frequency_table()`, builds a Huffman
/// tree out of `HuffmanNode`s using the frequency table with a `BinaryHeap`, 
/// then turns that huffman tree into a `HuffmanTable`.
fn build_huffman_table(freq_table:&[u64]) -> HuffmanTable {
    let mut node_heap:BinaryHeap<HuffmanNode> = BinaryHeap::new();
    for byte in 0..256 {
        if freq_table[byte] > 0{
            node_heap.push(HuffmanNode::leaf(byte as u8, freq_table[byte]));
        }
    }

    while node_heap.len() > 1{
        let left = node_heap.pop().unwrap();
        let right = node_heap.pop().unwrap();
        node_heap.push(HuffmanNode::node(left, right));
    }

    node_heap.pop().unwrap().leaves()
}

/// Prints the given `HuffmanTable`
fn print_huffman_table(huffman_table: &HuffmanTable) {
    for data in huffman_table{
        println!("Symbol: [{:x}] Level: [{}]", data.symbol, data.level);
    }
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
fn limit_huffman_table_code_sizes(huffman_table: &mut HuffmanTable, max_code_length:i32){

    assert!((huffman_table.len() as f32).log2().ceil() as i32 <= max_code_length, "Maximum code length of [{}] not large enough to store all [{}] symbols, needs length of at least [{}]", max_code_length, huffman_table.len(), (huffman_table.len() as f32).log2().ceil() as i32);

    let mut k = 0;
    let k_max:usize = (1 << max_code_length) - 1;

    for i in (0..huffman_table.len()).rev(){
        huffman_table[i].level = min(huffman_table[i].level, max_code_length);
        k += 1 << (max_code_length - huffman_table[i].level);
    }

    for i in (0..huffman_table.len()).rev(){
        if k <= k_max {
            break;
        }
        while huffman_table[i].level < max_code_length {
            huffman_table[i].level += 1;
            k -= 1 << (max_code_length - huffman_table[i].level);
        }
    }
    for i in 0..huffman_table.len(){
        while k + (1 << (max_code_length - huffman_table[i].level)) <= k_max {
            k += 1 << (max_code_length - huffman_table[i].level);
            huffman_table[i].level -= 1;
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
fn write_huffman_table(bitstream: &mut BitWriter, huffman_table: &HuffmanTable) {

    assert!(huffman_table.len() <= MAX_SYMBOLS, "The given Huffman table has too many symbols");

    bitstream.write_bits_u32(huffman_table.len() as u32, MAX_SYMBOLS_SIZE);

    let max_level = huffman_table.iter().max().unwrap().level; //Is this really necessary? I guess every little bit helps...
    bitstream.write_bits_u32(max_level as u32, MAX_SYMBOLS_SIZE);
    let bits_per_level = max((max_level as f32).log2().ceil() as usize, 1);

    for data in huffman_table{
        let symbol = data.symbol;
        let level = data.level;
        bitstream.write_bits_u32(symbol as u32, MAX_SYMBOLS_SIZE);
        bitstream.write_bits_u32(level as u32, bits_per_level);
    }
}

/// Builds a `HuffmanCodeMap` from a given `HuffmanTable`
/// 
/// Relies on the fact that all the nodes on a specific level can have codes
/// just by incrementing the code of the leaf next to it on the same level, if
/// that makes any sense.
fn build_huffman_code_map(huffman_table: &HuffmanTable) -> HuffmanCodeMap{
    let mut map:HuffmanCodeMap = vec![None; MAX_SYMBOLS];

    let mut code:HuffmanPath = 0;
    let mut last_level = -1;

    for data in huffman_table{
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
        map[symbol as usize] = Some((code, level as usize));
    }

    map
}

/// Prints a given `HuffmanCodeMap`
fn print_huffman_code_map(huffman_code_map: &HuffmanCodeMap) {
    for symbol in 0..MAX_SYMBOLS{
        if let Some((code, level)) = huffman_code_map[symbol]{
            print!("Symbol: [{:x}] Code:[", symbol);
            for i in (0..level).rev() {
                print!("{}", if (code & (1 << i)) > 0 {1} else {0});
            }
            println!("]");
        }
    }
}

/// Fills a symbol table and level table.
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
fn fill_huffman_symbol_maps(huffman_table: &HuffmanTable, symbol_table: &mut [HuffmanSymbol], level_table: &mut [i32], max_level: i32) {
    //let mut map:HuffmanSymbolMap = vec![HuffmanTableData { symbol:0, level:0 }; 1 << max_level];

    let mut code:HuffmanPath = 0;
    let mut last_level = -1;

    for data in huffman_table{
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
        let start_code = (code << (max_level - level)) as usize;
        let end_code = (start_code | ((1 << (max_level - level))-1)) as usize;
        symbol_table[start_code..=end_code].fill(symbol);
        level_table[start_code..=end_code].fill(level);
        // for i in 0..(1 << (max_level - level)){
        //     //println!("{:011b}",(code | (i << level)) as usize);
        //     map[i | (code << (max_level - level)) as usize] = HuffmanTableData { symbol, level };
        // }
        //map[code as usize] = Some(HuffmanTableData { symbol, level });
    }

}

// fn print_huffman_symbol_map(huffman_symbol_map: &HuffmanSymbolMap) {
//     for code in 0..MAX_SYMBOLS{
//         if let Some(data) = huffman_symbol_map[code]{
//             let symbol = data.symbol;
//             let level = data.level;
//             print!("Code: [");
//             for i in (0..level).rev() {
//                 print!("{}", if (code & (1 << i)) > 0 {1} else {0});
//             }
//             println!("] Symbol: [{:x}]", symbol);
//         }
//     }
// }


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
pub fn encode_bytes_huffman(bytes: &[u8], chunk_size:usize, max_path_size:i32) -> Vec<u8> {

    let mut bitstream = BitWriter::new();

    bitstream.write_bits_u32(max_path_size as u32, 8);

    for i in (0..bytes.len()).step_by(chunk_size){
        let chunk = &bytes[i..min(bytes.len(),i+chunk_size)];
        let curr_chunk_size = min(chunk_size, bytes.len() - i);

        let freq_table = build_frequency_table(chunk);
        //println!("Frequency table generated");
        let mut huffman_table = build_huffman_table(&freq_table);
        //println!("Huffman table generated");
        if max_path_size > 0{
            limit_huffman_table_code_sizes(&mut huffman_table, max_path_size);
        }
        //print_huffman_table(&huffman_table);
        //println!("{:?}",huffman_table);
        //println!("Max levels decreased");
        let map = build_huffman_code_map(&huffman_table);
        //print_huffman_code_map(&map);
        //println!("Huffman codes generated");
        bitstream.write_bits_u32(curr_chunk_size as u32, 24);
        //println!("{} {}", curr_chunk_size, bitstream);
        write_huffman_table(&mut bitstream, &huffman_table);
        //println!("{}",chunk_size, bitstream);
        //println!("Huffman table written");
        for byte in chunk{
            let (code, length) = map[*byte as usize].unwrap();
            bitstream.write_bits_u32(code, length);
        }
        bitstream.write_bits_u32(0, max_path_size as usize - map[ chunk[ chunk.len() - 1 ] as usize].unwrap().1);
        //println!("Number of symbols: [{}] Smallest code length: [{}] Largest code length: [{}]", huffman_table.len(),  huffman_table.iter().min().unwrap().level,  huffman_table.iter().max().unwrap().level);
    }
    //println!("File encoded");

    bitstream.get_bytes()

}

///Decodes a slice of bytes encoded using Huffman encoding.
/// 
/// WARNING: I don't know what this does if the encoded bytes weren't created
/// using my `encode_bytes_huffman` function. Therefore, I'd advise you don't
/// use it on anything not created using this function.
pub fn decode_bytes_huffman(encoded_bytes: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut bitstream = BitReader::new(encoded_bytes);

    let total_max_level = bitstream.read_bits_into_u32(8).unwrap() as i32;
    let mut symbol_table:Vec<HuffmanSymbol> = vec![0; 1 << total_max_level];
    let mut level_table:Vec<i32> = vec![0; 1 << total_max_level];

    while bitstream.remaining_bits() > 24 {
        //println!("{}",bitstream.bits_remaining());

        let chunk_size = bitstream.read_bits_into_u32(24).unwrap() as usize;
        let symbol_num = bitstream.read_bits_into_u32(8).unwrap();
        let max_level = bitstream.read_bits_into_u32(8).unwrap() as i32;
        let bits_per_level = max((max_level as f32).log2().ceil() as usize,1);
        //println!("Preliminary data read");
        //println!("Chunk size: [{}] Symbol num: [{}] Max level: [{}]", chunk_size, symbol_num, max_level);

        let mut huffman_table:HuffmanTable = Vec::with_capacity(MAX_SYMBOLS);
        for _ in 0..symbol_num{
            let symbol = bitstream.read_bits_into_u8(8).unwrap();
            let level = bitstream.read_bits_into_u32(bits_per_level).unwrap() as i32;
            huffman_table.push(HuffmanTableData{ symbol, level });
        }
        //println!("Huffman table read");
        //print_huffman_table(&huffman_table);

        fill_huffman_symbol_maps(&huffman_table, &mut symbol_table, &mut level_table, total_max_level);
        //println!("Symbol map generated");
        //print_huffman_symbol_map(&symbol_map);

        // let mut bytes_decoded:usize = 0;
        // let mut path:u32 = bool_slice_to_u32(&bitstream.read_bits((min_level-1) as usize).unwrap());
        // let mut path_len:i32 = min_level - 1;

        // while bytes_decoded < chunk_size{
        //     let bit = bitstream.read_bit().unwrap();
        //     path <<= 1;
        //     path |= if bit {1} else {0};
        //     path_len += 1;
        //     if let Some(data) = symbol_map[path as usize] {
        //         if data.level == path_len {
        //             println!("{:011b} {}",path,data.level);
        //             bytes.push(data.symbol);
        //             bytes_decoded += 1;
        //             path = bool_slice_to_u32(&bitstream.read_bits((min_level-1) as usize).unwrap());
        //             path_len = min_level-1;
        //         }
        //     }
        // }

        /*
        01111111100 10
        00000000101 4
        00000011100 5
        00000000000 3
        00000000100 4
        */

        let mut bytes_decoded:usize = 0;
        let mut path:u32 = 0;
        let mut bits_to_read:i32 = max_level;
        let mask:u32 = (1 << max_level) - 1;

        while bytes_decoded < chunk_size{
            //println!("{:011b} {}",path, bits_to_read);
            //let bit = bitstream.read_bit().unwrap();
            path |= bitstream.read_bits_into_u32(bits_to_read as usize).unwrap();
            //println!("{:011b}",path);
            //let data = symbol_map[path as usize];
            let symbol = symbol_table[path as usize];
            let level = level_table[path as usize];

            bytes.push(symbol);
            bytes_decoded += 1;
            path <<= level;
            path &= mask;
            bits_to_read = level;
        }
        //println!("Chunk decoded");

    }

    bytes
}