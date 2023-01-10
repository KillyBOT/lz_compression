use crate::bitstream::{BitReader, BitWriter};
use crate::huffman::{HuffmanSymbol, HuffmanPath, encode_bytes_huffman, decode_bytes_huffman};

type MatchLength = u32;
type MatchOffset = u32;

fn fast_log2_u32(v:u32) -> u32 {
    if v == 0 {return 0;}

    31 - v.leading_zeros()
}

/// Creates a `HuffmanSymbol` given a `MatchLength`.
/// The symbol is just equal to the length if the length is less than
/// 16, and equal to 12 + log2(length) if greater. This allows for more compact
/// symbols, as long as you also denote the extra bits using 
/// `extra_bits_from_length`, since 
fn huffman_symbol_from_length(length: MatchLength) -> HuffmanSymbol {
    if length < 16 {
        return length as HuffmanSymbol;
    }

    (12 + fast_log2_u32(length)) as HuffmanSymbol
}

fn extra_bits_from_length(length: MatchLength) -> HuffmanSymbol {
    (length - (1 << fast_log2_u32(length))) as HuffmanSymbol
}

fn huffman_symbol_from_offset(offset: MatchOffset) -> HuffmanSymbol {
    if offset < 2 {return offset as HuffmanSymbol;}

    (1 + fast_log2_u32(offset)) as HuffmanSymbol
}

#[cfg(test)]
mod tests{
    #[test]
    pub fn fast_log2_u32_test() {
        for num in [97758u32, 93658u32, 86636u32, 2623u32, 75343u32, 46835u32, 88189u32, 52233u32, 3907u32, 64476u32, 12861u32, 46261u32, 60695u32, 38029u32, 61168u32, 77655u32, 97815u32, 49371u32, 49118u32, 29138u32, 16451u32, 1607u32, 10663u32, 93815u32, 75146u32, 42807u32, 68381u32, 1063u32, 4990u32, 63744u32, 10137u32, 43254u32, 21171u32, 83417u32, 33270u32, 40106u32, 64128u32, 53782u32, 85183u32, 44082u32, 25309u32, 30617u32, 21117u32, 38969u32, 17873u32, 65888u32, 36684u32, 92196u32, 87049u32, 6080u32, 32493u32, 58124u32, 48669u32, 99194u32, 85970u32, 12357u32, 91229u32, 6132u32, 97989u32, 84058u32, 37744u32, 4562u32, 59294u32, 65236u32, 16571u32, 56115u32, 73037u32, 35545u32, 41656u32, 42748u32, 31338u32, 77068u32, 44765u32, 15301u32, 96648u32, 65541u32, 54921u32, 17102u32, 96644u32, 94647u32, 79280u32, 1456u32, 87750u32, 56138u32, 37030u32, 93057u32, 97301u32, 96730u32, 82814u32, 41527u32, 56546u32, 53109u32, 18328u32, 38914u32, 55667u32, 75697u32, 35198u32, 17457u32, 42311u32, 97125u32] {
            assert!((num as f32).log2().floor() as u32 == crate::lz::fast_log2_u32(num));
        }
    }
}