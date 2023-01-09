use std::fmt::{self};

const U64_MSB_MASK:u64 = 1 << 63;

pub struct BitWriter {
    bits_written_to_buffer:usize,
    buffer:u64,
    bytes:Vec<u8>
}

pub struct BitReader<'a> {
    buffer:u64,
    bits_in_buffer:usize,
    unused_bits_in_buffer:usize,
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
        let mut br = BitReader { buffer: 0, bits_in_buffer:0, unused_bits_in_buffer:64, bytes: bytes };
        br.refill();

        br
    }

    pub fn remaining_bits(&self) -> usize {
        (self.bytes.len() << 3) + self.bits_in_buffer
    }
    fn refill(&mut self) {
        while self.unused_bits_in_buffer >= 8 && self.bytes.len() > 0{
            let byte = self.bytes[0];
            self.bytes = &self.bytes[1..];
            self.bits_in_buffer += 8;
            self.unused_bits_in_buffer -= 8;
            self.buffer |= ((byte as u64) << self.unused_bits_in_buffer);
        }
    }

    pub fn read_bit(&mut self) -> Option<bool> {

        if self.remaining_bits() == 0 {
            return None;
        }

        //let bit = (self.bytes[self.bits_read >> 3] & (1 << (7 - (self.bits_read & 0b111)))) > 0;
        let bit = (self.buffer & U64_MSB_MASK) > 0;
        self.buffer <<= 1;
        self.bits_in_buffer -= 1;
        self.unused_bits_in_buffer += 1;
        self.refill();

        Some(bit)
    }

    pub fn read_bits_into_u8(&mut self, bit_num:usize) -> Option<u8> {

        assert!(bit_num <= 8, "Can only read up to 8 bits, attempted to read [{}] bits", bit_num);

        if bit_num > self.remaining_bits() {
            return None;
        }

        let bits = (self.buffer >> (64 - bit_num)) as u8;
        self.buffer <<= bit_num;
        self.bits_in_buffer -= bit_num;
        self.unused_bits_in_buffer += bit_num;
        self.refill();

        Some(bits)
    }

    pub fn read_bits_into_u32(&mut self, bit_num:usize) -> Option<u32> {

        assert!(bit_num <= 32, "Can only read up to 32 bits, attempted to read [{}] bits", bit_num);

        if bit_num > self.remaining_bits() {
            return None;
        }

        let bits = (self.buffer >> (64 - bit_num)) as u32;
        self.buffer <<= bit_num;
        self.bits_in_buffer -= bit_num;
        self.unused_bits_in_buffer += bit_num;
        self.refill();

        Some(bits)
    }

    // pub fn read_bits_into_usize(&mut self, bit_num:usize) -> Option<u8> {

    //     let usize_bit_size = std::mem::size_of::<usize>() << 3;
    //     assert!(bit_num <= usize_bit_size, "Can only read up to [{}] bits, attempted to read [{}] bits", usize_bit_size, bit_num);

    //     if bit_num > self.remaining_bits() {
    //         return None;
    //     }

    //     let bits = (self.buffer >> (64 - bit_num)) as u8;
    //     self.buffer <<= bit_num;
    //     self.bits_in_buffer -= bit_num;
    //     self.unused_bits_in_buffer += bit_num;
    //     self.refill();

    //     Some(bits)
    // }

    // pub fn read_bits(&mut self, bit_num:usize) -> Option<Vec<bool>> {

    //     if self.bits_read == self.bytes.len() << 3{
    //         return None;
    //     }

    //     let mut bool_vec = Vec::with_capacity(bit_num);

    //     for _ in 0..bit_num{
    //         if let Some(bit) = self.read_bit() {
    //             bool_vec.push(bit);
    //         }
    //     }

    //     Some(bool_vec)
    // }

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

}
