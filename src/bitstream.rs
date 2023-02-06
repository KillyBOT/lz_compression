use std::fmt::{self};

const U64_MSB_MASK:u64 = 1 << 63;

pub struct BitWriter {
    bits_written_to_buffer:usize,
    buffer:u64,
    bytes:Vec<u8>
}

pub struct BitReader<'a> {
    buffer:u64,
    remaining_bits: usize,
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
        let mut br = BitReader { buffer: 0, remaining_bits: bytes.len() << 3, bits_in_buffer:0, unused_bits_in_buffer:64, bytes: bytes };
        br.refill();

        br
    }

    pub fn remaining_bits(&self) -> usize {
        self.remaining_bits
    }

    fn refill(&mut self) {
        while self.unused_bits_in_buffer >= 8 && self.bytes.len() > 0{
            let byte = self.bytes[0];
            self.bytes = &self.bytes[1..];
            self.bits_in_buffer += 8;
            self.unused_bits_in_buffer -= 8;
            self.buffer |= (byte as u64) << self.unused_bits_in_buffer;
        }
        //println!("Bits in buffer: {}", self.bits_in_buffer);
    }

    fn print_buffer(&self) {
        let mut mask:u64 = 1 << 63;
        for i in 0..self.bits_in_buffer{
            print!("{}", if mask & self.buffer > 0 {1} else {0});
            mask >>= 1;
        }
        println!(" Size: {}", self.bits_in_buffer);
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
        self.remaining_bits -= 1;
        self.refill();

        Some(bit)
    }

    pub fn read_bits<T>(&mut self, bit_num:usize) -> Option<T> 
    where
    T: From<u64>{
        let max_bits = std::mem::size_of::<T>() << 3;
        assert!(bit_num <= max_bits, "Can only read up to [{max_bits}] bits, attempted to read [{bit_num}] bits");

        if self.remaining_bits == 0 {
            return None;
        } else if bit_num > self.remaining_bits {
            return self.read_bits::<T>(self.remaining_bits);
        } else if bit_num == 0{
            return Some(T::from(0));
        }

        let bits:T = T::from(self.buffer >> (64 - bit_num));
        self.buffer <<= bit_num;
        self.bits_in_buffer -= bit_num;
        self.unused_bits_in_buffer += bit_num;
        self.remaining_bits -= bit_num;

        self.refill();

        Some(bits)
    }

    pub fn read_bits_into_u8(&mut self, bit_num:usize) -> Option<u8> {

        assert!(bit_num <= 8, "Can only read up to 8 bits, attempted to read [{}] bits", bit_num);
        let remaining_bits = self.remaining_bits();
        //print!("Before read: ");
        //self.print_buffer();

        if remaining_bits == 0{
            return None;
        } else if bit_num > remaining_bits{
            return self.read_bits_into_u8(remaining_bits);
        } else if bit_num == 0 {
            return Some(0);
        }

        let bits = (self.buffer >> (64 - bit_num)) as u8;
        self.buffer <<= bit_num;
        self.bits_in_buffer -= bit_num;
        self.unused_bits_in_buffer += bit_num;
        self.remaining_bits -= bit_num;

        self.refill();

        Some(bits)
    }

    pub fn read_bits_into_u16(&mut self, bit_num:usize) -> Option<u16> {

        assert!(bit_num <= 16, "Can only read up to 16 bits, attempted to read [{}] bits", bit_num);
        //print!("Before read: ");
        //self.print_buffer();

        if self.remaining_bits == 0{
            return None;
        } else if bit_num > self.remaining_bits{
            return self.read_bits_into_u16(self.remaining_bits);
        } else if bit_num == 0 {
            return Some(0);
        }

        let bits = (self.buffer >> (64 - bit_num)) as u16;
        self.buffer <<= bit_num;
        self.bits_in_buffer -= bit_num;
        self.unused_bits_in_buffer += bit_num;
        self.remaining_bits -= bit_num;

        //print!("Before refill: ");
        //self.print_buffer();

        self.refill();

        //print!("After refill: ");
        //self.print_buffer();

        Some(bits)
    }

    pub fn read_bits_into_u32(&mut self, bit_num:usize) -> Option<u32> {

        assert!(bit_num <= 32, "Can only read up to 32 bits, attempted to read [{bit_num}] bits");

        if self.remaining_bits == 0{
            return None;
        } else if bit_num > self.remaining_bits{
            return self.read_bits_into_u32(self.remaining_bits);
        } else if bit_num == 0 {
            return Some(0);
        }

        let bits = (self.buffer >> (64 - bit_num)) as u32;
        self.buffer <<= bit_num;
        self.bits_in_buffer -= bit_num;
        self.unused_bits_in_buffer += bit_num;
        self.remaining_bits -= bit_num;

        //print!("Before refill: ");
        //self.print_buffer();

        self.refill();

        //print!("After refill: ");
        //self.print_buffer();

        Some(bits)
    }

    pub fn empty_bits(&mut self, bit_num:usize){
        
        if bit_num > self.remaining_bits {
            self.empty_bits(self.remaining_bits);
        }
        if bit_num > self.bits_in_buffer{
            self.empty_bits(self.bits_in_buffer);
            self.empty_bits(bit_num - self.bits_in_buffer);
        }

        self.buffer <<= bit_num;
        self.bits_in_buffer -= bit_num;
        self.unused_bits_in_buffer += bit_num;
        self.remaining_bits -= bit_num;

        self.refill();
    }

    pub fn read_bits_into_u32_with_shift(&mut self, bit_num:usize) -> Option<u32> {
        assert!(bit_num <= 32, "Can only read up to 32 bits, attempted to read [{bit_num}] bits");

        if self.remaining_bits == 0{
            return None;
        } else if bit_num > self.remaining_bits{
            let shift_amount = bit_num - self.remaining_bits;
            let val = self.read_bits_into_u32(self.remaining_bits).unwrap();
            return Some(val << shift_amount);
        } else if bit_num == 0 {
            return Some(0);
        }

        let bits = (self.buffer >> (64 - bit_num)) as u32;
        self.buffer <<= bit_num;
        self.bits_in_buffer -= bit_num;
        self.unused_bits_in_buffer += bit_num;

        //print!("Before refill: ");
        //self.print_buffer();

        self.refill();

        //print!("After refill: ");
        //self.print_buffer();

        Some(bits)
    }

    pub fn peek_bits_into_u32(&self, bit_num:usize) -> Option<u32> {

        assert!(bit_num <= 32, "Can only read up to 32 bits, attempted to read [{}] bits", bit_num);

        if self.remaining_bits == 0 {
            return None;
        } else if bit_num > self.remaining_bits{
            return self.peek_bits_into_u32(self.remaining_bits);
        } else if bit_num == 0 {
            return Some(0);
        }
            
        Some((self.buffer >> (64 - bit_num)) as u32)
    }

    pub fn peek_bits_into_u32_with_shift(&self, bit_num:usize) -> Option<u32> {
        assert!(bit_num <= 32, "Can only read up to 32 bits, attempted to read [{}] bits", bit_num);

        if self.remaining_bits == 0 {
            return None;
        } else if bit_num > self.remaining_bits{
            let shift_amount = bit_num - self.remaining_bits;
            let val = self.peek_bits_into_u32(self.remaining_bits).unwrap();
            return Some(val << shift_amount)
        } else if bit_num == 0 {
            return Some(0);
        }
            
        Some((self.buffer >> (64 - bit_num)) as u32)
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

    pub fn write_bits_u16(&mut self, data: u16, bit_num:usize){
        assert!(bit_num <= 16, "Number of bits must less than 32, given [{}] bits", bit_num);
        
        let mask = if bit_num == 16 {u16::MAX} else {(1 << bit_num) - 1};
        self.buffer |= ((data & mask) as u64) << (64 - self.bits_written_to_buffer - bit_num);
        self.bits_written_to_buffer += bit_num;
        self.flush();
    }
    pub fn write_bits_u32(&mut self, data: u32, bit_num:usize){
        assert!(bit_num <= 32, "Number of bits must less than 32, given [{}] bits", bit_num);
        
        let mask = if bit_num == 32 {u32::MAX} else {(1 << bit_num) - 1};
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

#[cfg(test)]
mod tests {
    use crate::bitstream::{BitWriter, BitReader};

    #[test]
    fn bit_reader_writer_test() {
        use rand::prelude::*;

        let val_num = 8192;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2123);

        let mut bit_num:usize = 0;
        let mut vals:Vec<u32> = Vec::with_capacity(val_num);
        let mut val_sizes:Vec<usize> = Vec::with_capacity(val_num);
        for _ in 0..val_num{
            let rand_len:usize = rng.gen_range(1..=32);
            let mask:u32 = if rand_len == 32 {u32::MAX} else {(1 << rand_len) - 1};
            let rand_val:u32 = rng.gen::<u32>() & mask;
            vals.push(rand_val);
            val_sizes.push(rand_len);
        }

        let mut writer = BitWriter::new();
        for i in 0..val_num{
            writer.write_bits_u32(vals[i], val_sizes[i]);
        }
        let bytes = writer.get_bytes();

        let mut reader = BitReader::new(&bytes);
        for i in 0..val_num{
            let read_val = reader.read_bits_into_u32(val_sizes[i]).unwrap();
            assert!(read_val == vals[i], "Val at position [{i}] was read/written incorrectly, {read_val} -> {}",vals[i]);
        }
    }
}