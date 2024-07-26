pub struct EncodeStatus {
    pub input_read: usize,
    pub output_written: usize,
}

const UNREADABLE_CHAR_REPLACEMENT: u16 = 0x2588;

pub fn str_utf8_to_utf16(input: &str, buffer: &mut [u16]) -> EncodeStatus {
    let input_bytes = input.as_bytes();
    let input_len = input_bytes.len();
    let mut current_pos = 0;
    let buffer_max_pos = buffer.len() - 1;  // Reserve 1 byte for NULL
    let mut buffer_pos = 0;

    while current_pos < input_len && buffer_pos < buffer_max_pos {
        match char_utf8_to_utf16(&input_bytes[current_pos..]) {
            EncodeCharStatus::EncodedChar { encoded_char, input_read } => {
                buffer[buffer_pos] = encoded_char;
                buffer_pos += 1;
                current_pos += input_read;
            },
            EncodeCharStatus::InsufficientInputBytes { input_read } => {
                buffer[buffer_pos] = UNREADABLE_CHAR_REPLACEMENT;
                buffer_pos += 1;
                current_pos += input_read;
            },
            EncodeCharStatus::SurrogatePair { input_read } => {
                buffer[buffer_pos] = UNREADABLE_CHAR_REPLACEMENT;
                buffer_pos += 1;
                current_pos += input_read;
            }
        }
    }

    buffer[buffer_pos] = 0;

    EncodeStatus { input_read: current_pos, output_written: buffer_pos }

}

enum EncodeCharStatus {
    EncodedChar {
        encoded_char: u16,
        input_read: usize,
    },
    SurrogatePair {
        input_read: usize,
    },
    InsufficientInputBytes {
        input_read: usize,
    },
}

const HIGH_BIT_MASK: u8 = 0b1000_0000;
const HIGH_TWO_BIT_MASK: u8 = 0b1100_0000;
const HIGH_THREE_BIT_MASK: u8 = 0b1110_0000;
const HIGH_FOUR_BIT_MASK: u8 = 0b1111_0000;
const LOW_FIVE_BIT_MASK: u8 = 0b0001_1111;
const LOW_SIX_BIT_MASK: u8 = 0b0011_1111;

fn char_utf8_to_utf16(input: &[u8]) -> EncodeCharStatus {
    let first_byte = input[0];

    if first_byte & HIGH_BIT_MASK == 0 {
        // Single byte char has the form 0xxx_xxxx
        return EncodeCharStatus::EncodedChar { 
            encoded_char: u16::from(first_byte), 
            input_read: 1,
        };
    } else if first_byte & HIGH_THREE_BIT_MASK == HIGH_TWO_BIT_MASK {
        // Two byte char has the form 110x_xxxx 10xx_xxxx
        if input.len() < 2 {
            // Expected more bytes than there are availiable. Should be Impossible.
            return EncodeCharStatus::InsufficientInputBytes { input_read: 1 };
        }

        let second_byte = input[1];
        let out_char = 
            u16::from(first_byte & LOW_FIVE_BIT_MASK) << 6 | 
            u16::from(second_byte & LOW_SIX_BIT_MASK);
        
        return EncodeCharStatus::EncodedChar { 
            encoded_char: out_char, 
            input_read: 2 
        };
    } else if first_byte & HIGH_FOUR_BIT_MASK == HIGH_THREE_BIT_MASK {
        // Three byte char has the form 1110_xxxx 10xx_xxxx 10xx_xxxx
        if input.len() < 3 {
            // Expected more bytes than there are availiable. Should be Impossible.
            return EncodeCharStatus::InsufficientInputBytes { input_read: input.len() };
        }
        let second_byte = input[1];
        let third_byte = input[2];

        let out_char = 
            u16::from(first_byte & LOW_FIVE_BIT_MASK) << 12 |
            u16::from(second_byte & LOW_SIX_BIT_MASK) << 6 |
            u16::from(third_byte & LOW_SIX_BIT_MASK);
        return EncodeCharStatus::EncodedChar { 
            encoded_char: out_char, 
            input_read: 3 
        };
    } else {
        // Four byte char has the form 1111_0xxx 10xx_xxxx 10xx_xxxx 10xx_xxxx
        if input.len() < 4 {
            // Expected more bytes than there are availiable. Should be Impossible.
            return EncodeCharStatus::InsufficientInputBytes { input_read: input.len() };
        }

        return EncodeCharStatus::SurrogatePair { input_read: 4 };
    }
}