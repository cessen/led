//! Encoding/decoding functions for big-endian UTF-16.
//!
//! Because both utf8 and utf16 can represent the entirety of unicode, the
//! only possible error is when invalid utf16 is encountered when decoding
//! to utf8.

use core;
use utils::{from_big_endian_u16, to_big_endian_u16};
use {DecodeError, DecodeResult, EncodeResult};

pub fn encode_from_utf8<'a>(input: &str, output: &'a mut [u8]) -> EncodeResult<'a> {
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
                input_i = offset;
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
            input_i = offset;
        } else {
            break;
        }
    }

    // Calculate how much of the input was consumed.
    input_i += 1;
    if input_i > input.len() {
        input_i = input.len();
    } else {
        while !input.is_char_boundary(input_i) {
            input_i += 1;
        }
    }

    Ok((input_i, &output[..output_i]))
}

pub fn decode_to_utf8<'a>(input: &[u8], output: &'a mut [u8]) -> DecodeResult<'a> {
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
                if !(input_i + 3) < input.len() {
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
