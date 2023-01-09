use std::fmt::{self};


pub struct BitWriter {
    bits_written_to_buffer:usize,
    buffer:u64,
    bytes:Vec<u8>
}

pub struct BitReader<'a> {
    bits_read:usize,
    bytes:&'a [u8]
}

impl fmt::Display for BitWriter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        // match &self.data {
        //     HuffmanNodeData::Node(left, right) => write!(f, "Frequency:[{}] Left:[{}] Right[{}]", self.freq, *left, *right),
        //     HuffmanNodeData::Leaf(symbol) => write!(f, "Frequency:[{}] Symbol:[{:x}]", self.freq, symbol)
        // }
        let mut repr:String = String::new();
        repr.push_str(format!("Bits written:[{}]\n", self.total_bits_written()).as_str());
        for byte in &self.bytes[0..self.bytes.len()]{
            repr.push_str(format!("{:08b} ",*byte).as_str());
        }
        for i in 0..self.bits_written_to_buffer{
            repr.push_str(format!("{}",( self.buffer >> (63-i)) & 1).as_str());

        }

        write!(f,"{}",repr)
        
    }
}

impl<'a> Iterator for BitReader<'a>{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_bit()
    }
}

impl<'a> BitReader<'a>{
    pub fn new(bytes: &'a [u8]) -> Self {
        BitReader { bits_read: 0, bytes: bytes }
    }

    pub fn bits_remaining(&self) -> usize {
        (self.bytes.len() << 3) - self.bits_read
    }

    pub fn read_bit(&mut self) -> Option<bool> {

        if self.bits_read == self.bytes.len() << 3{
            return None;
        }

        let bit = (self.bytes[self.bits_read >> 3] & (1 << (7 - (self.bits_read & 0b111)))) > 0;
        self.bits_read += 1;
        
        Some(bit)
    }
    pub fn read_bits(&mut self, bit_num:usize) -> Option<Vec<bool>> {

        if self.bits_read == self.bytes.len() << 3{
            return None;
        }

        let mut bool_vec = Vec::with_capacity(bit_num);

        for _ in 0..bit_num{
            if let Some(bit) = self.read_bit() {
                bool_vec.push(bit);
            }
        }

        Some(bool_vec)
    }

}

impl BitWriter {
    pub fn new() -> Self{
        BitWriter { bits_written_to_buffer: 0, buffer:0, bytes: Vec::new()}
    }

    pub fn total_bits_written(&self) -> usize {
        (self.bytes.len() << 3) + self.bits_written_to_buffer
    }

    fn flush(&mut self) {
        while self.bits_written_to_buffer >= 8{
            self.bytes.push( (self.buffer>>56) as u8);
            self.buffer <<= 8;
            self.bits_written_to_buffer -= 8;
        }
    }

    // pub fn finish(&mut self) {
    //     self.flush();
    //     let bits_remaining = 8 - self.bits_written_to_buffer;
    //     self.buffer <<= bits_remaining;
    //     self.bytes.push((self.buffer >> 56) as u8);
    //     self.buffer = 0;
    //     self.bits_written_to_buffer = 0;
    // }

    pub fn write_bits_u32(&mut self, data: u32, bit_num:usize){
        assert!(bit_num <= 32, "Number of bits must less than 32, given [{}] bits", bit_num);
        
        let mask = (1 << bit_num) - 1;
        self.buffer |= ((data & mask) as u64) << (64 - self.bits_written_to_buffer - bit_num);
        self.bits_written_to_buffer += bit_num;
        self.flush();
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut bytes = self.bytes.clone();
        if self.bits_written_to_buffer > 0 {
            bytes.push((self.buffer >> 56) as u8);
        }

        bytes.clone()
    }

    // pub fn write_bytes(&mut self, bytes: &Vec<u8>) {
    //     self.bits_written += bytes.len() * 8;
    //     self.bytes.extend(bytes);
    // }

}

pub fn bool_slice_to_u8(slice: &[bool]) -> u8{
    assert!(slice.len() <= 8, "Bool slice must be at most 8 bools long, slice given is {} bools long", slice.len());

    let mut byte:u8 = 0;

    for i in 0..slice.len(){
        if slice[i] {
            byte |= 1 << (slice.len() - i - 1);
        }
    }

    byte
}

pub fn bool_slice_to_u32(slice: &[bool]) -> u32 {
    assert!(slice.len() <= 32, "Bool slice must be at most 32 bools long, slice given is {} bools long", slice.len());

    let mut u_word:u32 = 0;

    for i in 0..slice.len(){
        if slice[i] {
            u_word |= 1 << (slice.len() - i - 1);
        }
    }

    u_word
}

pub fn bool_slice_to_usize(slice: &[bool]) -> usize {
    assert!(slice.len() <= std::mem::size_of::<usize>() << 3, "Bool slice must be at most {} bools long, slice given is {} bools long", std::mem::size_of::<usize>() << 3, slice.len());

    let mut size:usize = 0;

    for i in 0..slice.len(){
        if slice[i] {
            size |= 1 << (slice.len() - i - 1);
        }
    }

    size
}