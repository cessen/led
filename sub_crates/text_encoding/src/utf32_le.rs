//! Encoding/decoding functions for big-endian UTF-32.
//!
//! Because both utf8 and utf32 can represent the entirety of unicode, the
//! only possible error is when invalid utf32 is encountered when decoding
//! to utf8.

use core;
use utils::{from_little_endian_u32, to_little_endian_u32};
use {DecodeError, DecodeResult, EncodeResult};

pub fn encode_from_str<'a>(input: &str, output: &'a mut [u8]) -> EncodeResult<'a> {
    // Do the encode.
    let mut input_i = 0;
    let mut output_i = 0;
    for (offset, c) in input.char_indices() {
        if (output_i + 3) < output.len() {
            let mut code = to_little_endian_u32(c as u32);
            output[output_i] = code[0];
            output[output_i + 1] = code[1];
            output[output_i + 2] = code[2];
            output[output_i + 3] = code[3];
            output_i += 4;
            input_i = offset + 1;
        } else {
            break;
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

    // Loop through the input, getting 4 bytes at a time.
    let mut itr = input.chunks(4);
    while let Some(bytes) = itr.next() {
        if bytes.len() < 4 {
            break;
        }

        // Do the decode.
        if let Some(code) = core::char::from_u32(from_little_endian_u32([
            bytes[0], bytes[1], bytes[2], bytes[3],
        ])) {
            // Encode to utf8.
            let mut buf = [0u8; 4];
            let s = code.encode_utf8(&mut buf);
            if (output_i + s.len()) > output.len() {
                break;
            }
            output[output_i..(output_i + s.len())].copy_from_slice(s.as_bytes());

            // Update our counters.
            input_i += 4;
            output_i += s.len();
        } else {
            // Error: invalid codepoint.
            return Err(DecodeError {
                error_range: (input_i, input_i + 4),
                output_bytes_written: output_i,
            });
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
        let text = "こんにちは！";
        let mut buf = [0u8; 3];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 0);
        assert_eq!(encoded, &[]);
    }

    #[test]
    fn encode_02() {
        let text = "こんにちは！";
        let mut buf = [0u8; 4];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 3);
        assert_eq!(encoded, &[0x53, 0x30, 0x00, 0x00]);
    }

    #[test]
    fn encode_03() {
        let text = "こんにちは！";
        let mut buf = [0u8; 7];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 3);
        assert_eq!(encoded, &[0x53, 0x30, 0x00, 0x00]);
    }

    #[test]
    fn encode_04() {
        let text = "😺😼";
        let mut buf = [0u8; 3];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 0);
        assert_eq!(encoded, &[]);
    }

    #[test]
    fn encode_05() {
        let text = "😺😼";
        let mut buf = [0u8; 4];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(encoded, &[0x3A, 0xF6, 0x01, 0x00]);
    }

    #[test]
    fn encode_06() {
        let text = "😺😼";
        let mut buf = [0u8; 7];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(encoded, &[0x3A, 0xF6, 0x01, 0x00]);
    }

    #[test]
    fn decode_01() {
        let data = [
            0x53, 0x30, 0x00, 0x00, 0x93, 0x30, 0x00, 0x00, 0x6B, 0x30, 0x00, 0x00, 0x61, 0x30,
            0x00, 0x00, 0x6F, 0x30, 0x00, 0x00, 0x01, 0xFF, 0x00, 0x00,
        ]; // "こんにちは！"
        let mut buf = [0u8; 2];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 0);
        assert_eq!(decoded, "");
    }

    #[test]
    fn decode_02() {
        let data = [
            0x53, 0x30, 0x00, 0x00, 0x93, 0x30, 0x00, 0x00, 0x6B, 0x30, 0x00, 0x00, 0x61, 0x30,
            0x00, 0x00, 0x6F, 0x30, 0x00, 0x00, 0x01, 0xFF, 0x00, 0x00,
        ]; // "こんにちは！"
        let mut buf = [0u8; 3];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "こ");
    }

    #[test]
    fn decode_03() {
        let data = [
            0x53, 0x30, 0x00, 0x00, 0x93, 0x30, 0x00, 0x00, 0x6B, 0x30, 0x00, 0x00, 0x61, 0x30,
            0x00, 0x00, 0x6F, 0x30, 0x00, 0x00, 0x01, 0xFF, 0x00, 0x00,
        ]; // "こんにちは！"
        let mut buf = [0u8; 5];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "こ");
    }

    #[test]
    fn decode_04() {
        let data = [0x3A, 0xF6, 0x01, 0x00, 0x3C, 0xF6, 0x01, 0x00]; // "😺😼"
        let mut buf = [0u8; 3];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 0);
        assert_eq!(decoded, "");
    }

    #[test]
    fn decode_05() {
        let data = [0x3A, 0xF6, 0x01, 0x00, 0x3C, 0xF6, 0x01, 0x00]; // "😺😼"
        let mut buf = [0u8; 4];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "😺");
    }

    #[test]
    fn decode_06() {
        let data = [0x3A, 0xF6, 0x01, 0x00, 0x3C, 0xF6, 0x01, 0x00]; // "😺😼"
        let mut buf = [0u8; 7];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "😺");
    }

    #[test]
    fn decode_07() {
        let data = [0x3A, 0xF6, 0x01, 0x00, 0x3C, 0xF6, 0x01]; // "😺😼" with last byte chopped off.
        let mut buf = [0u8; 64];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "😺");
    }

    #[test]
    fn decode_08() {
        let data = [0x3A, 0xF6, 0x01, 0x00, 0x3C, 0xF6]; // "😺😼" with last 2 bytes chopped off.
        let mut buf = [0u8; 64];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "😺");
    }

    #[test]
    fn decode_09() {
        let data = [0x3A, 0xF6, 0x01, 0x00, 0x3C]; // "😺😼" with last 3 bytes chopped off.
        let mut buf = [0u8; 64];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "😺");
    }

    #[test]
    fn decode_error_01() {
        let data = [
            0x00, 0x00, 0x11, 0x00, 0x93, 0x30, 0x00, 0x00, 0x6B, 0x30, 0x00, 0x00, 0x61, 0x30,
            0x00, 0x00, 0x6F, 0x30, 0x00, 0x00, 0x01, 0xFF, 0x00, 0x00,
        ]; // "こんにちは！" with an error on the first char (value out of range)
        let mut buf = [0u8; 2];
        assert_eq!(
            decode_to_str(&data, &mut buf),
            Err(DecodeError {
                error_range: (0, 4),
                output_bytes_written: 0,
            })
        );
    }

    #[test]
    fn decode_error_02() {
        let data = [
            0x00, 0xD8, 0x00, 0x00, 0x93, 0x30, 0x00, 0x00, 0x6B, 0x30, 0x00, 0x00, 0x61, 0x30,
            0x00, 0x00, 0x6F, 0x30, 0x00, 0x00, 0x01, 0xFF, 0x00, 0x00,
        ]; // "こんにちは！" with an error on the first char (value in surrogate range)
        let mut buf = [0u8; 2];
        assert_eq!(
            decode_to_str(&data, &mut buf),
            Err(DecodeError {
                error_range: (0, 4),
                output_bytes_written: 0,
            })
        );
    }

    #[test]
    fn decode_error_03() {
        let data = [
            0xFF, 0xDF, 0x00, 0x00, 0x93, 0x30, 0x00, 0x00, 0x6B, 0x30, 0x00, 0x00, 0x61, 0x30,
            0x00, 0x00, 0x6F, 0x30, 0x00, 0x00, 0x01, 0xFF, 0x00, 0x00,
        ]; // "こんにちは！" with an error on the first char (value in surrogate range)
        let mut buf = [0u8; 64];
        assert_eq!(
            decode_to_str(&data, &mut buf),
            Err(DecodeError {
                error_range: (0, 4),
                output_bytes_written: 0,
            })
        );
    }

    #[test]
    fn decode_error_04() {
        let data = [
            0x53, 0x30, 0x00, 0x00, 0x00, 0x00, 0x11, 0x00, 0x6B, 0x30, 0x00, 0x00, 0x61, 0x30,
            0x00, 0x00, 0x6F, 0x30, 0x00, 0x00, 0x01, 0xFF, 0x00, 0x00,
        ]; // "こんにちは！" with an error on the second char (value out of range)
        let mut buf = [0u8; 64];
        assert_eq!(
            decode_to_str(&data, &mut buf),
            Err(DecodeError {
                error_range: (4, 8),
                output_bytes_written: 3,
            })
        );
        assert_eq!(&buf[..3], &[0xE3, 0x81, 0x93]);
    }

    #[test]
    fn decode_error_05() {
        let data = [
            0x53, 0x30, 0x00, 0x00, 0x00, 0xD8, 0x00, 0x00, 0x6B, 0x30, 0x00, 0x00, 0x61, 0x30,
            0x00, 0x00, 0x6F, 0x30, 0x00, 0x00, 0x01, 0xFF, 0x00, 0x00,
        ]; // "こんにちは！" with an error on the second char (value in surrogate range)
        let mut buf = [0u8; 64];
        assert_eq!(
            decode_to_str(&data, &mut buf),
            Err(DecodeError {
                error_range: (4, 8),
                output_bytes_written: 3,
            })
        );
        assert_eq!(&buf[..3], &[0xE3, 0x81, 0x93]);
    }

    #[test]
    fn decode_error_06() {
        let data = [
            0x53, 0x30, 0x00, 0x00, 0xFF, 0xDF, 0x00, 0x00, 0x6B, 0x30, 0x00, 0x00, 0x61, 0x30,
            0x00, 0x00, 0x6F, 0x30, 0x00, 0x00, 0x01, 0xFF, 0x00, 0x00,
        ]; // "こんにちは！" with an error on the second char (value in surrogate range)
        let mut buf = [0u8; 64];
        assert_eq!(
            decode_to_str(&data, &mut buf),
            Err(DecodeError {
                error_range: (4, 8),
                output_bytes_written: 3,
            })
        );
        assert_eq!(&buf[..3], &[0xE3, 0x81, 0x93]);
    }
}
