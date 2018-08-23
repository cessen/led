//! Encoding/decoding functions for Windows-1252.

use core;
use {DecodeError, DecodeResult, EncodeError, EncodeResult};

pub fn encode_from_str<'a>(input: &str, output: &'a mut [u8]) -> EncodeResult<'a> {
    // Do the encode.
    let mut input_i = 0;
    let mut output_i = 0;
    for (offset, c) in input.char_indices() {
        if output_i >= output.len() {
            break;
        }
        if let Some(byte) = encode_table(c) {
            output[output_i] = byte;
            output_i += 1;
            input_i = offset + 1;
        } else {
            return Err(EncodeError {
                character: c,
                error_range: (offset, offset + c.len_utf8()),
                output_bytes_written: output_i,
            });
        }
    }

    // Calculate how much of the input was consumed.
    if input_i > input.len() {
        input_i = input.len();
    } else {
        while !input.is_char_boundary(input_i) {
            input_i += 1;
        }
    }

    Ok((input_i, &output[..output_i]))
}

pub fn decode_to_str<'a>(input: &[u8], output: &'a mut [u8]) -> DecodeResult<'a> {
    let mut input_i = 0;
    let mut output_i = 0;
    for &byte in input.iter() {
        if byte < 0x80 {
            // 1-byte case
            if output_i >= output.len() {
                break;
            }
            output[output_i] = byte;
            input_i += 1;
            output_i += 1;
        } else if byte < 0xA0 {
            // Use lookup table.
            let code = DECODE_TABLE[byte as usize - 0x80];
            if code == '�' {
                // Error: undefined byte.
                return Err(DecodeError {
                    error_range: (input_i, input_i + 1),
                    output_bytes_written: output_i,
                });
            }
            // Encode to utf8
            let mut buf = [0u8; 4];
            let s = code.encode_utf8(&mut buf);
            if (output_i + s.len()) > output.len() {
                break;
            }
            output[output_i..(output_i + s.len())].copy_from_slice(s.as_bytes());
            input_i += 1;
            output_i += s.len();
        } else {
            // Non-lookup-table 2-byte case
            if (output_i + 1) >= output.len() {
                break;
            }
            output[output_i] = 0b11000000 | (byte >> 6);
            output[output_i + 1] = 0b10000000 | (byte & 0b00111111);
            input_i += 1;
            output_i += 2;
        }
    }

    Ok((input_i, unsafe {
        core::str::from_utf8_unchecked(&output[..output_i])
    }))
}

// Maps unicode to windows-1252.
//
// Returns `None` for characters not in windows-1252.
#[inline(always)]
fn encode_table(code: char) -> Option<u8> {
    if (code as u32) < 0x80 || ((code as u32) > 0x9F && (code as u32) <= 0xFF) {
        return Some(code as u8);
    }
    match code {
        '\u{20AC}' => Some(0x80),
        '\u{201A}' => Some(0x82),
        '\u{0192}' => Some(0x83),
        '\u{201E}' => Some(0x84),
        '\u{2026}' => Some(0x85),
        '\u{2020}' => Some(0x86),
        '\u{2021}' => Some(0x87),
        '\u{02C6}' => Some(0x88),
        '\u{2030}' => Some(0x89),
        '\u{0160}' => Some(0x8A),
        '\u{2039}' => Some(0x8B),
        '\u{0152}' => Some(0x8C),
        '\u{017D}' => Some(0x8E),
        '\u{2018}' => Some(0x91),
        '\u{2019}' => Some(0x92),
        '\u{201C}' => Some(0x93),
        '\u{201D}' => Some(0x94),
        '\u{2022}' => Some(0x95),
        '\u{2013}' => Some(0x96),
        '\u{2014}' => Some(0x97),
        '\u{02DC}' => Some(0x98),
        '\u{2122}' => Some(0x99),
        '\u{0161}' => Some(0x9A),
        '\u{203A}' => Some(0x9B),
        '\u{0153}' => Some(0x9C),
        '\u{017E}' => Some(0x9E),
        '\u{0178}' => Some(0x9F),
        _ => None,
    }
}

// Maps the range 0x80-0x9F in windows-1252 to unicode.  The remaining
// characters in windows-1252 match unicode.
//
// The '�'s stand in for codes not defined in windows-1252, and should be
// be treated as an error when encountered.
const DECODE_TABLE: [char; 32] = [
    '\u{20AC}', '�', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '�', '\u{017D}', '�', '�',
    '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}', '\u{02DC}',
    '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '�', '\u{017E}', '\u{0178}',
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_01() {
        let text = "Hello world!";
        let mut buf = [0u8; 0];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 0);
        assert_eq!(encoded, &[]);
    }

    #[test]
    fn encode_02() {
        let text = "Hello world!";
        let mut buf = [0u8; 1];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 1);
        assert_eq!(encoded, "H".as_bytes());
    }

    #[test]
    fn encode_03() {
        let text = "Hello world!";
        let mut buf = [0u8; 2];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 2);
        assert_eq!(encoded, "He".as_bytes());
    }

    #[test]
    fn encode_04() {
        let text = "Hello world!";
        let mut buf = [0u8; 64];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 12);
        assert_eq!(encoded, "Hello world!".as_bytes());
    }

    #[test]
    fn encode_05() {
        let text = "Hello world!こ";
        let mut buf = [0u8; 12];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 12);
        assert_eq!(encoded, "Hello world!".as_bytes());
    }

    #[test]
    fn decode_01() {
        let data = [
            0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        ]; // "Hello world!"
        let mut buf = [0u8; 0];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 0);
        assert_eq!(decoded, "");
    }

    #[test]
    fn decode_02() {
        let data = [
            0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        ]; // "Hello world!"
        let mut buf = [0u8; 1];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 1);
        assert_eq!(decoded, "H");
    }

    #[test]
    fn decode_03() {
        let data = [
            0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        ]; // "Hello world!"
        let mut buf = [0u8; 2];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 2);
        assert_eq!(decoded, "He");
    }

    #[test]
    fn decode_04() {
        let data = [
            0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        ]; // "Hello world!"
        let mut buf = [0u8; 64];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 12);
        assert_eq!(decoded, "Hello world!");
    }

    #[test]
    fn decode_05() {
        let data = [
            0x80, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8E, 0x91,
            0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9E, 0x9F,
        ]; // "€‚ƒ„…†‡ˆ‰Š‹ŒŽ‘’“”•–—˜™š›œžŸ", all of the non-latin1 matching characters.
        let mut buf = [0u8; 128];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 27);
        assert_eq!(
            decoded,
            "€‚ƒ„…†‡ˆ‰Š‹ŒŽ‘’“”•–—˜™š›œžŸ"
        );
    }

    #[test]
    fn encode_error_01() {
        let text = "こello world!";
        let mut buf = [0u8; 64];
        assert_eq!(
            encode_from_str(text, &mut buf),
            Err(EncodeError {
                character: 'こ',
                error_range: (0, 3),
                output_bytes_written: 0,
            })
        );
    }

    #[test]
    fn encode_error_02() {
        let text = "\u{0085}ello world!";
        let mut buf = [0u8; 64];
        assert_eq!(
            encode_from_str(text, &mut buf),
            Err(EncodeError {
                character: '\u{0085}',
                error_range: (0, 2),
                output_bytes_written: 0,
            })
        );
    }

    #[test]
    fn encode_error_03() {
        let text = "Hこllo world!";
        let mut buf = [0u8; 64];
        assert_eq!(
            encode_from_str(text, &mut buf),
            Err(EncodeError {
                character: 'こ',
                error_range: (1, 4),
                output_bytes_written: 1,
            })
        );
    }

    #[test]
    fn encode_error_04() {
        let text = "H\u{0085}llo world!";
        let mut buf = [0u8; 64];
        assert_eq!(
            encode_from_str(text, &mut buf),
            Err(EncodeError {
                character: '\u{0085}',
                error_range: (1, 3),
                output_bytes_written: 1,
            })
        );
    }

    #[test]
    fn encode_error_05() {
        let text = "Heこlo world!";
        let mut buf = [0u8; 3];
        assert_eq!(
            encode_from_str(text, &mut buf),
            Err(EncodeError {
                character: 'こ',
                error_range: (2, 5),
                output_bytes_written: 2,
            })
        );
    }

    #[test]
    fn encode_error_06() {
        let text = "He\u{0085}lo world!";
        let mut buf = [0u8; 3];
        assert_eq!(
            encode_from_str(text, &mut buf),
            Err(EncodeError {
                character: '\u{0085}',
                error_range: (2, 4),
                output_bytes_written: 2,
            })
        );
    }

    #[test]
    fn decode_error_01() {
        let data = [
            0x48, 0x81, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        ]; // "Hello world!" with an error on the second byte (undefined byte).
        let mut buf = [0u8; 64];
        let error = decode_to_str(&data, &mut buf);
        assert_eq!(
            error,
            Err(DecodeError {
                error_range: (1, 2),
                output_bytes_written: 1,
            })
        );
    }

    #[test]
    fn decode_error_02() {
        let data = [
            0x48, 0x8D, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        ]; // "Hello world!" with an error on the second byte (undefined byte).
        let mut buf = [0u8; 64];
        let error = decode_to_str(&data, &mut buf);
        assert_eq!(
            error,
            Err(DecodeError {
                error_range: (1, 2),
                output_bytes_written: 1,
            })
        );
    }

    #[test]
    fn decode_error_03() {
        let data = [
            0x48, 0x8F, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        ]; // "Hello world!" with an error on the second byte (undefined byte).
        let mut buf = [0u8; 64];
        let error = decode_to_str(&data, &mut buf);
        assert_eq!(
            error,
            Err(DecodeError {
                error_range: (1, 2),
                output_bytes_written: 1,
            })
        );
    }

    #[test]
    fn decode_error_04() {
        let data = [
            0x48, 0x90, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        ]; // "Hello world!" with an error on the second byte (undefined byte).
        let mut buf = [0u8; 64];
        let error = decode_to_str(&data, &mut buf);
        assert_eq!(
            error,
            Err(DecodeError {
                error_range: (1, 2),
                output_bytes_written: 1,
            })
        );
    }

    #[test]
    fn decode_error_05() {
        let data = [
            0x48, 0x9D, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        ]; // "Hello world!" with an error on the second byte (undefined byte).
        let mut buf = [0u8; 64];
        let error = decode_to_str(&data, &mut buf);
        assert_eq!(
            error,
            Err(DecodeError {
                error_range: (1, 2),
                output_bytes_written: 1,
            })
        );
    }
}
