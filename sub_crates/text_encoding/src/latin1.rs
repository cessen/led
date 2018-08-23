//! Encoding/decoding functions for ISO/IEC 8859-1 (or "latin1"), which
//! conveniently happens to map 1-to-1 to the first 256 unicode scalar values.
//!
//! Because latin1 is a single-byte encoding where all bytes are valid,
//! decoding cannot fail.  However, encoding will fail with scalar values
//! greater than 255.

use core;
use {DecodeResult, EncodeError, EncodeResult};

pub fn encode_from_str<'a>(input: &str, output: &'a mut [u8]) -> EncodeResult<'a> {
    // Do the encode.
    let mut input_i = 0;
    let mut output_i = 0;
    for (offset, c) in input.char_indices() {
        if output_i >= output.len() {
            break;
        }
        if c as u32 > 255 {
            return Err(EncodeError {
                character: c,
                error_range: (offset, offset + c.len_utf8()),
                output_bytes_written: output_i,
            });
        }
        output[output_i] = c as u8;
        output_i += 1;
        input_i = offset + 1;
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
        if byte <= 127 {
            // 1-byte case
            if output_i >= output.len() {
                break;
            }
            output[output_i] = byte;
            input_i += 1;
            output_i += 1;
        } else {
            // 2-byte case
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
        let data = "Hello world!".as_bytes();
        let mut buf = [0u8; 0];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 0);
        assert_eq!(decoded, "");
    }

    #[test]
    fn decode_02() {
        let data = "Hello world!".as_bytes();
        let mut buf = [0u8; 1];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 1);
        assert_eq!(decoded, "H");
    }

    #[test]
    fn decode_03() {
        let data = "Hello world!".as_bytes();
        let mut buf = [0u8; 2];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 2);
        assert_eq!(decoded, "He");
    }

    #[test]
    fn decode_04() {
        let data = "Hello world!".as_bytes();
        let mut buf = [0u8; 64];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 12);
        assert_eq!(decoded, "Hello world!");
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
    fn encode_error_03() {
        let text = "Heこlo world!";
        let mut buf = [0u8; 64];
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
    fn encode_error_04() {
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
}
