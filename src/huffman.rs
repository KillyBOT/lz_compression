use std::collections::{BinaryHeap, HashMap};
use std::cmp::{Ordering, min, max};
use std::fmt::{self};
use crate::bitstream::{BitStream};

const MAX_SYMBOLS:usize = 256;
const MAX_SYMBOLS_SIZE:usize = 8;
const MAX_CODE_LENGTH:i32 = 11;

pub type HuffmanSymbol = u8;
type HuffmanPath = u64;

#[derive(Debug, Eq)]
pub struct HuffmanTableData {
    pub symbol:u8,
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

    //println!("{:?}", node_heap);

    while node_heap.len() > 1{
        let left = node_heap.pop().unwrap();
        let right = node_heap.pop().unwrap();
        node_heap.push(HuffmanNode::node(left, right));
    }

    node_heap.pop().unwrap().leaves()
}

fn limit_huffman_table_code_sizes(huffman_table: &mut [HuffmanTableData]){
    let mut k = 0;
    let k_max:usize = (1 << MAX_CODE_LENGTH) - 1;

    for i in (0..huffman_table.len()).rev(){
        huffman_table[i].level = min(huffman_table[i].level, MAX_CODE_LENGTH);
        k += 1 << (MAX_CODE_LENGTH - huffman_table[i].level);
    }

    for i in (0..huffman_table.len()).rev(){
        if k <= k_max {
            break;
        }
        while huffman_table[i].level < MAX_CODE_LENGTH {
            huffman_table[i].level += 1;
            k -= 1 << (MAX_CODE_LENGTH - huffman_table[i].level);
        }
    }
    for i in 0..huffman_table.len(){
        while k + (1 << (MAX_CODE_LENGTH - huffman_table[i].level)) <= k_max {
            k += 1 << (MAX_CODE_LENGTH - huffman_table[i].level);
            huffman_table[i].level -= 1;
        }
    }
}

fn write_huffman_table(bitstream: &mut BitStream, huffman_table: &[HuffmanTableData]) {

    assert!(huffman_table.len() <= MAX_SYMBOLS, "The given Huffman table has too many symbols");

    bitstream.write_bits_u64(huffman_table.len() as u64, MAX_SYMBOLS_SIZE);

    let max_level = huffman_table.iter().max().unwrap().level; //Is this really necessary? I guess every little bit helps...
    bitstream.write_bits_u64(max_level as u64, MAX_SYMBOLS_SIZE);
    let bits_per_level = max((max_level as f32).log2().ceil() as usize, 1);

    for data in huffman_table{
        let symbol = data.symbol;
        let level = data.level;
        bitstream.write_bits_u64(symbol as u64, MAX_SYMBOLS_SIZE);
        bitstream.write_bits_u64(level as u64, bits_per_level);
    }
}

fn build_huffman_code_map(huffman_table: &[HuffmanTableData]) -> HuffmanCodeMap{
    let mut map:HuffmanCodeMap = vec![None; MAX_SYMBOLS];

    let mut code:u64 = 0;
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
        map[symbol as usize] = Some((code, level as usize));
    }

    map
}

pub fn encode_bytes(bytes: &[u8]) -> BitStream {
    let mut bitstream = BitStream::new();
    let freq_table = build_frequency_table(bytes);
    //println!("Frequency table generated");
    let mut huffman_table = build_huffman_table(&freq_table);
    //println!("Huffman table generated");
    limit_huffman_table_code_sizes(&mut huffman_table);
    //println!("Max levels decreased");
    let map = build_huffman_code_map(&huffman_table);
    //println!("Huffman codes generated");

    write_huffman_table(&mut bitstream, &huffman_table);
    //println!("Huffman table written");

    for byte in bytes{
        let (code, length) = map[*byte as usize].unwrap();
        bitstream.write_bits_u64(code, length);
    }
    //println!("File encoded");

    bitstream

}