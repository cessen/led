//! Encoding/decoding functions for big-endian UTF-16.
//!
//! Because both utf8 and utf16 can represent the entirety of unicode, the
//! only possible error is when invalid utf16 is encountered when decoding
//! to utf8.

use core;
use utils::{from_big_endian_u16, to_big_endian_u16};
use {DecodeError, DecodeResult, EncodeResult};

pub fn encode_from_str<'a>(input: &str, output: &'a mut [u8]) -> EncodeResult<'a> {
    // Do the encode.
    let mut input_i = 0;
    let mut output_i = 0;
    for (offset, c) in input.char_indices() {
        let mut code = c as u32;
        if code <= 0xFFFF {
            // One code unit
            if (output_i + 1) < output.len() {
                let val = to_big_endian_u16(code as u16);
                output[output_i] = val[0];
                output[output_i + 1] = val[1];
                output_i += 2;
                input_i = offset + 1;
            } else {
                break;
            }
        } else if (output_i + 3) < output.len() {
            // Two code units
            code -= 0x10000;
            let first = to_big_endian_u16(0xD800 | ((code >> 10) as u16));
            let second = to_big_endian_u16(0xDC00 | ((code as u16) & 0x3FF));
            output[output_i] = first[0];
            output[output_i + 1] = first[1];
            output[output_i + 2] = second[0];
            output[output_i + 3] = second[1];
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

    // Loop through the input, getting 2 bytes at a time.
    let mut itr = input.chunks(2);
    while let Some(bytes) = itr.next() {
        if bytes.len() < 2 {
            break;
        }

        // Decode to scalar value.
        let code = {
            let code_1 = from_big_endian_u16([bytes[0], bytes[1]]);
            if code_1 < 0xD800 || code_1 > 0xDFFF {
                // Single code unit.
                unsafe { core::char::from_u32_unchecked(code_1 as u32) }
            } else if (code_1 & 0xFC00) == 0xDC00 {
                // Error: orphaned second half of a surrogate pair.
                return Err(DecodeError {
                    error_range: (input_i, input_i + 2),
                    output_bytes_written: output_i,
                });
            } else {
                // Two code units.

                // Get the second code unit, if possible.
                if (input_i + 3) >= input.len() {
                    break;
                }
                let bytes_2 = itr.next().unwrap();
                let code_2 = from_big_endian_u16([bytes_2[0], bytes_2[1]]);
                if (code_2 & 0xFC00) != 0xDC00 {
                    // Error: second half is not valid surrogate.
                    return Err(DecodeError {
                        error_range: (input_i, input_i + 2),
                        output_bytes_written: output_i,
                    });
                }

                unsafe {
                    core::char::from_u32_unchecked(
                        (((code_1 as u32 - 0xD800) << 10) | (code_2 as u32 - 0xDC00)) + 0x10000,
                    )
                }
            }
        };

        // Encode to utf8.
        let mut buf = [0u8; 4];
        let s = code.encode_utf8(&mut buf);
        if (output_i + s.len()) > output.len() {
            break;
        }
        output[output_i..(output_i + s.len())].copy_from_slice(s.as_bytes());

        // Update our counters.
        input_i += code.len_utf16() * 2;
        output_i += s.len();
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
        let mut buf = [0u8; 1];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 0);
        assert_eq!(encoded, &[]);
    }

    #[test]
    fn encode_02() {
        let text = "こんにちは！";
        let mut buf = [0u8; 2];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 3);
        assert_eq!(encoded, &[0x30, 0x53]);
    }

    #[test]
    fn encode_03() {
        let text = "こんにちは！";
        let mut buf = [0u8; 3];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 3);
        assert_eq!(encoded, &[0x30, 0x53]);
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
        assert_eq!(encoded, &[0xD8, 0x3D, 0xDE, 0x3A]);
    }

    #[test]
    fn encode_06() {
        let text = "😺😼";
        let mut buf = [0u8; 7];
        let (consumed_count, encoded) = encode_from_str(text, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(encoded, &[0xD8, 0x3D, 0xDE, 0x3A]);
    }

    #[test]
    fn decode_01() {
        let data = [
            0x30, 0x53, 0x30, 0x93, 0x30, 0x6B, 0x30, 0x61, 0x30, 0x6F, 0xFF, 0x01,
        ]; // "こんにちは！"
        let mut buf = [0u8; 2];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 0);
        assert_eq!(decoded, "");
    }

    #[test]
    fn decode_02() {
        let data = [
            0x30, 0x53, 0x30, 0x93, 0x30, 0x6B, 0x30, 0x61, 0x30, 0x6F, 0xFF, 0x01,
        ]; // "こんにちは！"
        let mut buf = [0u8; 3];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 2);
        assert_eq!(decoded, "こ");
    }

    #[test]
    fn decode_03() {
        let data = [
            0x30, 0x53, 0x30, 0x93, 0x30, 0x6B, 0x30, 0x61, 0x30, 0x6F, 0xFF, 0x01,
        ]; // "こんにちは！"
        let mut buf = [0u8; 5];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 2);
        assert_eq!(decoded, "こ");
    }

    #[test]
    fn decode_04() {
        let data = [0xD8, 0x3D, 0xDE, 0x3A, 0xD8, 0x3D, 0xDE, 0x3C]; // "😺😼"
        let mut buf = [0u8; 3];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 0);
        assert_eq!(decoded, "");
    }

    #[test]
    fn decode_05() {
        let data = [0xD8, 0x3D, 0xDE, 0x3A, 0xD8, 0x3D, 0xDE, 0x3C]; // "😺😼"
        let mut buf = [0u8; 4];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "😺");
    }

    #[test]
    fn decode_06() {
        let data = [0xD8, 0x3D, 0xDE, 0x3A, 0xD8, 0x3D, 0xDE, 0x3C]; // "😺😼"
        let mut buf = [0u8; 7];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "😺");
    }

    #[test]
    fn decode_07() {
        let data = [0xD8, 0x3D, 0xDE, 0x3A, 0xD8, 0x3D]; // "😺😼" with last codepoint chopped off.
        let mut buf = [0u8; 64];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "😺");
    }

    #[test]
    fn decode_08() {
        let data = [0xD8, 0x3D, 0xDE, 0x3A, 0xD8, 0x3D, 0xDE]; // "😺😼" with last byte chopped off.
        let mut buf = [0u8; 64];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "😺");
    }

    #[test]
    fn decode_09() {
        let data = [0xD8, 0x3D, 0xDE, 0x3A, 0xD8]; // "😺😼" with last 3 bytes chopped off.
        let mut buf = [0u8; 64];
        let (consumed_count, decoded) = decode_to_str(&data, &mut buf).unwrap();
        assert_eq!(consumed_count, 4);
        assert_eq!(decoded, "😺");
    }

    #[test]
    fn decode_error_01() {
        let data = [
            0xDE, 0x3A, 0x30, 0x93, 0x30, 0x6B, 0x30, 0x61, 0x30, 0x6F, 0xFF, 0x01,
        ]; // "こんにちは！" with an error on the first char (end surrogate)
        let mut buf = [0u8; 2];
        let error = decode_to_str(&data, &mut buf);
        assert_eq!(
            error,
            Err(DecodeError {
                error_range: (0, 2),
                output_bytes_written: 0,
            })
        );
    }

    #[test]
    fn decode_error_02() {
        let data = [
            0x30, 0x53, 0xDE, 0x3A, 0x30, 0x6B, 0x30, 0x61, 0x30, 0x6F, 0xFF, 0x01,
        ]; // "こんにちは！" with an error on the second char (end surrogate)
        let mut buf = [0u8; 3];
        let error = decode_to_str(&data, &mut buf);
        assert_eq!(
            error,
            Err(DecodeError {
                error_range: (2, 4),
                output_bytes_written: 3,
            })
        );
    }

    #[test]
    fn decode_error_03() {
        let data = [
            0x30, 0x53, 0x30, 0x93, 0x30, 0x6B, 0xDE, 0x3A, 0x30, 0x6F, 0xFF, 0x01,
        ]; // "こんにちは！" with an error on the fourth char (end surrogate)
        let mut buf = [0u8; 64];
        let error = decode_to_str(&data, &mut buf);
        assert_eq!(
            error,
            Err(DecodeError {
                error_range: (6, 8),
                output_bytes_written: 9,
            })
        );
    }

    #[test]
    fn decode_error_04() {
        let data = [
            0xD8, 0x3D, 0x30, 0x93, 0x30, 0x6B, 0x30, 0x61, 0x30, 0x6F, 0xFF, 0x01,
        ]; // "こんにちは！" with an error on the first char (start surrogate)
        let mut buf = [0u8; 2];
        let error = decode_to_str(&data, &mut buf);
        assert_eq!(
            error,
            Err(DecodeError {
                error_range: (0, 2),
                output_bytes_written: 0,
            })
        );
    }

    #[test]
    fn decode_error_05() {
        let data = [
            0x30, 0x53, 0xD8, 0x3D, 0x30, 0x6B, 0x30, 0x61, 0x30, 0x6F, 0xFF, 0x01,
        ]; // "こんにちは！" with an error on the second char (start surrogate)
        let mut buf = [0u8; 3];
        let error = decode_to_str(&data, &mut buf);
        assert_eq!(
            error,
            Err(DecodeError {
                error_range: (2, 4),
                output_bytes_written: 3,
            })
        );
    }

    #[test]
    fn decode_error_06() {
        let data = [
            0x30, 0x53, 0x30, 0x93, 0x30, 0x6B, 0xD8, 0x3D, 0x30, 0x6F, 0xFF, 0x01,
        ]; // "こんにちは！" with an error on the fourth char (start surrogate)
        let mut buf = [0u8; 64];
        let error = decode_to_str(&data, &mut buf);
        assert_eq!(
            error,
            Err(DecodeError {
                error_range: (6, 8),
                output_bytes_written: 9,
            })
        );
    }
}
