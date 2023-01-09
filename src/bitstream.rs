use std::fmt::{self};


pub struct BitWriter {
    bits_written:usize,
    buffer:u64,
    bits_remaining:usize,
    bytes:Vec<u8>
}

pub struct BitReader {
    bits_read:usize,
    num_of_bits:usize,
    bytes:Vec<u8>
}

impl fmt::Display for BitWriter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        // match &self.data {
        //     HuffmanNodeData::Node(left, right) => write!(f, "Frequency:[{}] Left:[{}] Right[{}]", self.freq, *left, *right),
        //     HuffmanNodeData::Leaf(symbol) => write!(f, "Frequency:[{}] Symbol:[{:x}]", self.freq, symbol)
        // }
        let mut repr:String = String::new();
        repr.push_str(format!("Bits written:[{}]\n", self.bits_written).as_str());
        for i in 0..self.bytes_written(){
            let byte = self.bytes[i];
            repr.push_str(format!("{:08b} ",byte).as_str());
        }

        for i in (0..(64 - self.bits_remaining)).rev(){
            repr.push_str(format!("{}",(self.buffer >> i) & 1).as_str());
            if i > 0 && (i % 8) == 0{
                repr.push_str(" ");
            }
        }

        write!(f,"{}",repr)
        
    }
}

impl Iterator for BitReader{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits_read >= self.num_of_bits{
            return None
        }
        let byte = self.bytes_read();
        let shift = 7 - (self.bits_read & 0b111);
        self.bits_read += 1;

        Some(((self.bytes[byte] >> shift) & 1) == 1)
    }
}

impl BitReader {
    pub fn new(bytes: &[u8], num_of_bits: usize) -> Self {
        BitReader { bits_read: 0, num_of_bits: num_of_bits, bytes: bytes.to_vec() }
    }

    pub fn bytes_read(&self) -> usize {
        self.bits_read >> 3
    }
}

impl BitWriter {
    pub fn new() -> Self{
        BitWriter { bits_written: 0, buffer:0, bits_remaining:64, bytes: Vec::new()}
    }

    // pub fn from_bytes(bytes: &[u8]) -> Self {
    //     BitWriter { bits_written: 8 * bytes.len(), bits_read: 0, buffer:0, bits_remaining:64, bytes: bytes.to_vec()}
    // }
    
    pub fn bytes_written(&self) -> usize {
        self.bits_written >> 3
    }

    fn flush(&mut self) {
        if self.bits_remaining < 64{
            let bytes_written_to_buffer = ((63 - self.bits_remaining) >> 3) + 1; //floor((x-1)/8)+1, x = # of bits written

            if self.bits_remaining < 64 {
                self.buffer <<= self.bits_remaining;
            }

            let bytes_in_buffer = self.buffer.to_be_bytes();
            for byte in &bytes_in_buffer[0..bytes_written_to_buffer]{
                self.bytes.push(*byte);
                self.bits_written += 8;
            }

            self.buffer = 0;
            self.bits_remaining = 64;
        }
    }


    pub fn write_bit(&mut self, bit: bool) {
        self.buffer <<= 1;
        self.buffer |= if bit {1} else {0};
        self.bits_written += 1;
        self.bits_remaining -= 1;

        if self.bits_remaining == 0{
            self.flush();
        }
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

    pub fn get_bytes(&self) -> (Vec<u8>, usize) {
        (self.bytes.clone(), self.bits_written)
    }

    // pub fn write_bytes(&mut self, bytes: &Vec<u8>) {
    //     self.bits_written += bytes.len() * 8;
    //     self.bytes.extend(bytes);
    // }

}