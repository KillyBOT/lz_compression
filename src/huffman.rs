use std::collections::{BinaryHeap};
use std::cmp::{Ordering, min, max};
use std::fmt::{self};
use crate::bitstream::{BitWriter, BitReader, bool_slice_to_u8, bool_slice_to_usize, bool_slice_to_u32};

const MAX_SYMBOLS:usize = 256;
const MAX_SYMBOLS_SIZE:usize = 8;
const CHUNK_SIZE:usize = 1 << 18;

pub type HuffmanSymbol = u8;
type HuffmanPath = u32;

#[derive(Debug, Eq, Clone, Copy)]
pub struct HuffmanTableData {
    pub symbol:HuffmanSymbol,
    pub level:i32
}

#[derive(Debug)]
enum HuffmanNodeData {
    Node(Box<HuffmanNode>, Box<HuffmanNode>),
    Leaf(HuffmanSymbol)
}

#[derive(Debug)]
struct HuffmanNode {
    freq: u64,
    data: HuffmanNodeData
}

type HuffmanTable = Vec<HuffmanTableData>;
type HuffmanCodeMap = Vec<Option<(HuffmanPath,usize)>>;
type HuffmanSymbolMap = Vec<Option<HuffmanTableData>>;

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
    pub fn leaf(symbol: u8, freq: u64) -> Self{
        HuffmanNode{
            freq,
            data:HuffmanNodeData::Leaf(symbol)
        }
    }

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


fn build_frequency_table(bytes: &[u8]) -> Vec<u64> {
    let mut freq_vec = Vec::with_capacity(MAX_SYMBOLS);
    freq_vec.resize(MAX_SYMBOLS, 0);

    for byte in bytes{
        freq_vec[*byte as usize] += 1;
    }

    freq_vec
}

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

fn print_huffman_table(huffman_table: &[HuffmanTableData]) {
    for data in huffman_table{
        println!("Symbol: [{:x}] Level: [{}]", data.symbol, data.level);
    }
}

fn limit_huffman_table_code_sizes(huffman_table: &mut [HuffmanTableData], max_code_length:i32){
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

fn write_huffman_table(bitstream: &mut BitWriter, huffman_table: &[HuffmanTableData]) {

    assert!(huffman_table.len() <= MAX_SYMBOLS, "The given Huffman table has too many symbols");

    bitstream.write_bits_u32(huffman_table.len() as u32, MAX_SYMBOLS_SIZE);

    let max_level = huffman_table.iter().max().unwrap().level; //Is this really necessary? I guess every little bit helps...
    bitstream.write_bits_u32(max_level as u32, MAX_SYMBOLS_SIZE);
    let bits_per_level = (max_level as f32).log2().ceil() as usize;

    for data in huffman_table{
        let symbol = data.symbol;
        let level = data.level;
        bitstream.write_bits_u32(symbol as u32, MAX_SYMBOLS_SIZE);
        bitstream.write_bits_u32(level as u32, bits_per_level);
    }
}

fn build_huffman_code_map(huffman_table: &[HuffmanTableData]) -> HuffmanCodeMap{
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

fn build_huffman_symbol_map(huffman_table: &[HuffmanTableData], max_level: i32) -> HuffmanSymbolMap{
    let mut map:HuffmanSymbolMap = vec![None; 1 << max_level];

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
        map[code as usize] = Some(HuffmanTableData { symbol, level });
    }

    map
}

fn print_huffman_symbol_map(huffman_symbol_map: &HuffmanSymbolMap) {
    for code in 0..MAX_SYMBOLS{
        if let Some(data) = huffman_symbol_map[code]{
            let symbol = data.symbol;
            let level = data.level;
            print!("Code: [");
            for i in (0..level).rev() {
                print!("{}", if (code & (1 << i)) > 0 {1} else {0});
            }
            println!("] Symbol: [{:x}]", symbol);
        }
    }
}

pub fn encode_bytes_huffman(bytes: &[u8], chunk_size:usize, max_path_size:i32) -> Vec<u8> {

    let mut bitstream = BitWriter::new();

    for i in (0..bytes.len()).step_by(chunk_size){
        let chunk = &bytes[i..min(bytes.len(),i+chunk_size)];
        let curr_chunk_size = min(CHUNK_SIZE, bytes.len() - i);

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
        //println!("Number of symbols: [{}] Smallest code length: [{}] Largest code length: [{}]", huffman_table.len(),  huffman_table.iter().min().unwrap().level,  huffman_table.iter().max().unwrap().level);
    }
    //println!("File encoded");

    bitstream.get_bytes()

}

pub fn decode_bytes_huffman(encoded_bytes: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut bitstream = BitReader::new(encoded_bytes);

    while bitstream.bits_remaining() > 24 {
        //println!("{}",bitstream.bits_remaining());

        let chunk_size = bool_slice_to_usize(&bitstream.read_bits(24).unwrap());
        let symbol_num = bool_slice_to_usize(&bitstream.read_bits(8).unwrap());
        let max_level = bool_slice_to_u32(&bitstream.read_bits(8).unwrap()) as i32;
        let bits_per_level = (max_level as f32).log2().ceil() as usize;
        //println!("Chunk size: [{}] Symbol num: [{}] Max level: [{}]", chunk_size, symbol_num, max_level);

        let mut huffman_table:HuffmanTable = Vec::with_capacity(MAX_SYMBOLS);
        for i in 0..symbol_num{
            let symbol = bool_slice_to_u8(&bitstream.read_bits(8).unwrap());
            let level = bool_slice_to_u32(&bitstream.read_bits(bits_per_level).unwrap()) as i32;
            huffman_table.push(HuffmanTableData{ symbol, level });
        }
        //print_huffman_table(&huffman_table);

        let symbol_map = build_huffman_symbol_map(&huffman_table, max_level);
        //print_huffman_symbol_map(&symbol_map);

        let mut bytes_decoded:usize = 0;
        let mut path:u32 = 0;
        let mut path_len:i32 = 0;

        while bytes_decoded < chunk_size{
            let bit = bitstream.read_bit().unwrap();
            path <<= 1;
            path |= if bit {1} else {0};
            path_len += 1;
            if let Some(data) = symbol_map[path as usize] {
                if data.level == path_len {
                    bytes.push(data.symbol);
                    bytes_decoded += 1;
                    path = 0;
                    path_len = 0;

                }
            }
        }

    }

    bytes
}