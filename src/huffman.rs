use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::fmt;

const BYTES_MASK:usize = !0b111;

#[derive(Debug)]
pub enum HuffmanNodeData {
    Node(Box<HuffmanNode>, Box<HuffmanNode>),
    Leaf(u8)
}

#[derive(Debug)]
pub struct HuffmanNode {
    freq: usize,
    data: HuffmanNodeData
}

pub struct Bitstream {
    bits_written:usize,
    bits_read:usize,
    bytes:Vec<u8>
}

impl Bitstream {
    pub fn new() -> Self{
        Bitstream { bits_written: 0, bits_read: 0, bytes: Vec::new()}
    }

    pub fn bytes_read(&self) -> usize {
        self.bits_read >> 3
    }
    
    pub fn bytes_written(&self) -> usize {
        self.bits_written >> 3
    }

    pub fn read_bit_front(&mut self, bit:u8) -> Option<u8> {
        if self.bits_written <= self.bits_read {
            return None;
        }

        let byte = self.bytes_read();
        let shift = 7 - self.bits_read & 0b111;
        self.bits_read += 1;

        Some((self.bytes[byte] >> shift) & 1)
    }

    pub fn write_bit(&mut self, bit: u8) {
        if self.bits_written & 0b111 == 0 {
            self.bytes.push(0);
        }

        let end = self.bytes_written();
        self.bytes[end] <<= 1;
        self.bytes[end] |= bit;
        self.bits_written += 1;
    }

    pub fn write_bytes(&mut self, bytes: &Vec<u8>) {
        self.bits_written += bytes.len() * 8;
        self.bytes.extend(bytes);
    }

    pub fn flush(&mut self) -> &[u8]{
        let bits_left = 8 - self.bits_written & 0b111;
        let end = self.bytes.len() - 1;
        self.bytes[end] <<= bits_left;

        self.bytes.as_slice()
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

    pub fn leaves(&self, leaves: &mut Vec<(u8,u8)>){
        match &self.data{
            HuffmanNodeData::Node(left, right) => {
                left.leaves(leaves);
                right.leaves(leaves);
            },
            HuffmanNodeData::Leaf(symbol) => {
                leaves.push((self.freq as u8, *symbol));
            }
        }
    }

    pub fn walk(&mut self, dist: usize) {
        match &mut self.data {
            HuffmanNodeData::Leaf(_) => {self.freq = dist}
            HuffmanNodeData::Node(left, right) => {
                left.walk(dist + 1);
                right.walk(dist + 1);
            }
        }
    }
}

pub fn build_frequency_table(bytes: &[u8]) -> Vec<usize> {
    let mut freq_vec = vec![0; 256];

    for byte in bytes{
        freq_vec[*byte as usize] += 1;
    }

    freq_vec
}

pub fn build_huffman_tree(freq_table:Vec<usize>) -> HuffmanNode {
    let mut node_heap:BinaryHeap<HuffmanNode> = BinaryHeap::new();
    for byte in 0..256 {
        if freq_table[byte] > 0{
            node_heap.push(HuffmanNode::leaf(byte as u8, freq_table[byte]));
        }
    }

    println!("{:?}", node_heap);

    while node_heap.len() > 1{
        let left = node_heap.pop().unwrap();
        let right = node_heap.pop().unwrap();
        node_heap.push(HuffmanNode::node(left, right));
    }

    let mut tree = node_heap.pop().unwrap();

    tree
}

pub fn build_codes_from_tree(tree: &HuffmanNode) -> Vec<(u8, u8)> {
    let mut nodes_sorted = Vec::new();
    tree.leaves(&mut nodes_sorted);
    nodes_sorted.sort();
    
    let mut codes = vec![(0,0);256];

    let mut code:u8 = 0;
    let mut level:u8 = 0;

    codes
}