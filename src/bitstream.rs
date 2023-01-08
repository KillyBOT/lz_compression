use std::fmt::{self};


pub struct BitStream {
    bits_written:usize,
    bits_read:usize,
    buffer:u64,
    bits_remaining:usize,
    bytes:Vec<u8>
}


impl fmt::Display for BitStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        // match &self.data {
        //     HuffmanNodeData::Node(left, right) => write!(f, "Frequency:[{}] Left:[{}] Right[{}]", self.freq, *left, *right),
        //     HuffmanNodeData::Leaf(symbol) => write!(f, "Frequency:[{}] Symbol:[{:x}]", self.freq, symbol)
        // }
        let mut repr:String = String::new();
        repr.push_str(format!("Bits written:[{}] Bits read:[{}]\n", self.bits_written, self.bits_read).as_str());
        for i in self.bytes_read()..self.bytes_written(){
            let byte = self.bytes[i];
            repr.push_str(format!("{:08b} ",byte).as_str());
        }

        for i in (0..(64 - self.bits_remaining)).rev(){
            repr.push_str(format!("{}",(self.buffer >> i) & 1).as_str());
        }


        write!(f,"{}",repr)
        
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
        BitStream { bits_written: 0, bits_read: 0, buffer:0, bits_remaining:64, bytes: Vec::new()}
    }

    // pub fn from_bytes(bytes: &[u8]) -> Self {
    //     BitStream { bits_written: 8 * bytes.len(), bits_read: 0, buffer:0, bits_remaining:64, bytes: bytes.to_vec()}
    // }

    pub fn bytes_read(&self) -> usize {
        self.bits_read >> 3
    }
    
    pub fn bytes_written(&self) -> usize {
        self.bits_written >> 3
    }

    fn flush(&mut self) {

        if self.bits_remaining < 64 {
            self.buffer <<= self.bits_remaining;
            self.bits_written += self.bits_remaining;
        }
        for i in (0..self.bits_remaining).step_by(8).rev(){
            self.bytes.push( (self.buffer >> i) as u8);
        }
        self.buffer = 0;
        self.bits_remaining = 64;
    }

    pub fn write_bits_u64(&mut self, data: u64, bit_num:usize){
        assert!(0 < bit_num && bit_num <= 64, "Number of bits must be between 1 and 64, given [{}] bits", bit_num);

        if bit_num > self.bits_remaining{
            let rest_of_bits_num = bit_num - self.bits_remaining;
            let first = data >> rest_of_bits_num;
            let rest = data & ((1 << rest_of_bits_num) - 1);
            self.write_bits_u64(first, self.bits_remaining);
            self.write_bits_u64(rest, rest_of_bits_num);
        } else {
            let mask:u64 = (1 << bit_num) - 1;
            self.buffer <<= bit_num;
            self.buffer |= data & mask;
            self.bits_written += bit_num;
            self.bits_remaining -= bit_num;
            if self.bits_remaining == 0{
                self.flush();
            }
        }
    }

    // pub fn write_bytes(&mut self, bytes: &Vec<u8>) {
    //     self.bits_written += bytes.len() * 8;
    //     self.bytes.extend(bytes);
    // }

}

