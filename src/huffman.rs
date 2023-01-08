use std::collections::{BinaryHeap, HashMap};
use std::cmp::{Ordering, min, max};
use std::fmt::{self};

const BITS_MASK:usize = 0b111;
const MAX_SYMBOLS:usize = 256;
const MAX_SYMBOLS_SIZE:usize = 8;
const MAX_CODE_LENGTH:usize = 11;

pub type HuffmanSymbol = u8;

#[derive(Debug, Eq)]
pub struct HuffmanTableData {
    pub symbol:u8,
    pub level:usize
}

#[derive(Debug)]
pub enum HuffmanNodeData {
    Node(Box<HuffmanNode>, Box<HuffmanNode>),
    Leaf(HuffmanSymbol)
}

#[derive(Debug)]
pub struct HuffmanCodeData {
    pub path: usize,
    pub length: usize
}

#[derive(Debug)]
pub struct HuffmanNode {
    freq: usize,
    data: HuffmanNodeData
}

pub struct BitStream {
    bits_written:usize,
    bits_read:usize,
    bytes:Vec<u8>
}
pub type HuffmanTable = Vec<HuffmanTableData>;
pub type HuffmanCodeMap = HashMap<HuffmanSymbol, HuffmanCodeData>;

impl fmt::Display for BitStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        // match &self.data {
        //     HuffmanNodeData::Node(left, right) => write!(f, "Frequency:[{}] Left:[{}] Right[{}]", self.freq, *left, *right),
        //     HuffmanNodeData::Leaf(symbol) => write!(f, "Frequency:[{}] Symbol:[{:x}]", self.freq, symbol)
        // }
        let mut rep:String = String::new();
        rep.push_str(format!("Bits written:[{}] Bits read:[{}]\n", self.bits_written, self.bits_read).as_str());
        for i in self.bytes_read()..self.bytes_written(){
            let byte = self.bytes[i];
            rep.push_str(format!("{:08b} ",byte).as_str());
        }

        if self.bits_written > self.bytes_written() * 8 {
            let unused_bits = 8 - (self.bits_written & BITS_MASK);
        
            println!("{}",unused_bits);
            for i in (unused_bits..=7).rev() {
                rep.push_str(format!("{}",(self.bytes[self.bytes_written()] >> i) & 1).as_str());
            }
        }

        write!(f,"{}",rep)
        
    }
}

impl Iterator for BitStream {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits_read < self.bits_written {
            let last_byte = self.bytes[self.bytes_read()];
            let bits_left = 7 - ((self.bits_read) & 0b111);
            self.bits_read += 1;
            return Some( (last_byte >> bits_left) & 1 );
        }

        None
    }
}

impl BitStream {
    pub fn new() -> Self{
        BitStream { bits_written: 0, bits_read: 0, bytes: Vec::new()}
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        BitStream { bits_written: 8 * bytes.len(), bits_read: 0, bytes: bytes.to_vec()}
    }

    pub fn bytes_read(&self) -> usize {
        self.bits_read >> 3
    }
    
    pub fn bytes_written(&self) -> usize {
        self.bits_written >> 3
    }

    pub fn write_bit(&mut self, bit: u8) {
        if self.bits_written & BITS_MASK == 0 {
            self.bytes.push(0);
        }

        let end = self.bytes_written();
        self.bytes[end] <<= 1;
        self.bytes[end] |= bit;
        self.bits_written += 1;
    }

    pub fn write_bits_u8(&mut self, byte: u8, bit_num:usize){

        assert!(0 < bit_num && bit_num <= 8, "Number of bits must be between 1 and 8, given [{}] bits", bit_num);
        
        let size_left = 8 - (self.bits_written & BITS_MASK);

        if size_left < bit_num{

            let first_byte = byte >> size_left;
            let second_byte = byte ^ (first_byte << size_left);
            self.write_bits_u8(first_byte, size_left);
            self.write_bits_u8(second_byte, size_left);
        } else {

            if self.bits_written & BITS_MASK == 0 {
                self.bytes.push(0);
            }
    
            let end = self.bytes_written();
            if bit_num == 8{
                self.bytes[end] = byte;
            } else {
                self.bytes[end] <<= bit_num;
                self.bytes[end] |= byte;
            }
            self.bits_written += bit_num;
        }

        
    }

    pub fn write_bits_usize(&mut self, data: usize, bit_num:usize){
        assert!(0 < bit_num && bit_num <= std::mem::size_of::<usize>() * 8, "Number of bits must be between 1 and [{}], given [{}] bits",std::mem::size_of::<usize>() * 8, bit_num);

        if bit_num > 8{
            self.write_bits_usize(data >> 8, bit_num - 8);
            self.write_bits_u8(data as u8, 8);
        } else {
            self.write_bits_u8(data as u8, bit_num);
        }
        
    }

    pub fn write_bytes(&mut self, bytes: &Vec<u8>) {
        self.bits_written += bytes.len() * 8;
        self.bytes.extend(bytes);
    }

    pub fn flush(&mut self) -> &[u8]{
        let bits_left = 8 - (self.bits_written & BITS_MASK);
        let end = self.bytes_written();
        self.bytes[end] <<= bits_left;

        self.bytes.as_slice()
    }
}



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
    pub fn leaf(symbol: u8, freq: usize) -> Self{
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

    fn leaves_helper(&self, leaves: &mut HuffmanTable, level: usize){
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



impl fmt::Display for HuffmanCodeData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        // match &self.data {
        //     HuffmanNodeData::Node(left, right) => write!(f, "Frequency:[{}] Left:[{}] Right[{}]", self.freq, *left, *right),
        //     HuffmanNodeData::Leaf(symbol) => write!(f, "Frequency:[{}] Symbol:[{:x}]", self.freq, symbol)
        // }
        let mut repr = String::new();
        repr.push_str(format!("Length:[{}] Path:[",self.length).as_str());
        for i in (0..self.length).rev() {
            repr.push_str(((self.path >> i) & 1).to_string().as_str());
        }

        write!(f, "{}]", repr)
        
    }
}

impl HuffmanCodeData {
    pub fn new(path: usize, length: usize) -> Self {
        HuffmanCodeData {
            path,
            length
        }
    }
}


pub fn build_frequency_table(bytes: &[u8]) -> Vec<usize> {
    let mut freq_vec = Vec::with_capacity(MAX_SYMBOLS);
    freq_vec.resize(MAX_SYMBOLS, 0);

    for byte in bytes{
        freq_vec[*byte as usize] += 1;
    }

    freq_vec
}

pub fn build_huffman_table(freq_table:&[usize]) -> HuffmanTable {
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

pub fn limit_huffman_table_code_sizes(huffman_table: &mut [HuffmanTableData]){
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

pub fn write_huffman_table(bitstream: &mut BitStream, huffman_table: &[HuffmanTableData]) {

    assert!(huffman_table.len() <= MAX_SYMBOLS, "The given Huffman table has too many symbols");

    bitstream.write_bits_u8(huffman_table.len() as u8, MAX_SYMBOLS_SIZE);

    let max_level = huffman_table.iter().max().unwrap().level; //Is this really necessary? I guess every little bit helps...
    bitstream.write_bits_usize(max_level, MAX_SYMBOLS_SIZE);
    let bits_per_level = max((max_level as f32).log2().ceil() as usize, 1);

    println!("{} {}", max_level, bits_per_level);

    for data in huffman_table{
        let symbol = data.symbol;
        let level = data.level;
        bitstream.write_bits_u8(symbol, MAX_SYMBOLS_SIZE);
        bitstream.write_bits_usize(level, bits_per_level);
    }
}

pub fn build_huffman_code_map(huffman_table: &[HuffmanTableData]) -> HuffmanCodeMap{
    let mut map:HuffmanCodeMap = HashMap::with_capacity(huffman_table.len());

    let mut code:usize = 0;
    let mut last_level_option:Option<usize> = None;

    for data in huffman_table{
        let symbol = data.symbol;
        let level = data.level as usize;

        if let Some(last_level) = last_level_option {
            if last_level != level {
                code += 1;
                code <<= level - last_level;
                last_level_option = Some(level);
            } else {
                code += 1;
            }
        } else {
            last_level_option = Some(level);
        }

        map.insert(symbol, HuffmanCodeData::new(code, level as usize));
    }

    map
}



pub fn encode_bytes(bytes: &[u8]) -> BitStream {
    let mut bitstream = BitStream::new();
    let freq_table = build_frequency_table(bytes);
    let mut huffman_table = build_huffman_table(&freq_table);
    limit_huffman_table_code_sizes(&mut huffman_table);
    let huffman_code_map = build_huffman_code_map(&huffman_table);

    write_huffman_table(&mut bitstream, &huffman_table);

    for byte in bytes{
        let data = huffman_code_map.get(byte).unwrap();
        bitstream.write_bits_usize(data.path, data.length);
    }

    bitstream

}