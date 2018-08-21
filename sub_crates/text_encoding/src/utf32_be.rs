//! Encoding/decoding functions for big-endian UTF-32.
//!
//! Because both utf8 and utf32 can represent the entirety of unicode, the
//! only possible error is when invalid utf32 is encountered when decoding
//! to utf8.

use std;
use {DecodeError, DecodeResult, EncodeResult};

fn to_big_endian(n: u32) -> [u8; 4] {
    use std::mem::transmute;
    let ptr = unsafe { transmute::<*const u32, *const u8>(&n as *const u32) };
    if cfg!(target_endian = "little") {
        unsafe { [*ptr.offset(3), *ptr.offset(2), *ptr.offset(1), *ptr] }
    } else {
        unsafe { [*ptr, *ptr.offset(1), *ptr.offset(2), *ptr.offset(3)] }
    }
}

fn from_big_endian(n: [u8; 4]) -> u32 {
    use std::mem::transmute;
    let mut x: u32 = 0;
    let ptr = unsafe { transmute::<*mut u32, *mut u8>(&mut x as *mut u32) };
    if cfg!(target_endian = "little") {
        unsafe {
            *ptr = n[3];
            *ptr.offset(1) = n[2];
            *ptr.offset(2) = n[1];
            *ptr.offset(3) = n[0];
        }
    } else {
        unsafe {
            *ptr = n[0];
            *ptr.offset(1) = n[1];
            *ptr.offset(2) = n[2];
            *ptr.offset(3) = n[3];
        }
    }
    x
}

pub fn encode_from_utf8<'a>(input: &str, output: &'a mut [u8]) -> EncodeResult<'a> {
    // Do the encode.
    let mut input_i = 0;
    let mut output_i = 0;
    for (offset, c) in input.char_indices() {
        if (output_i + 3) < output.len() {
            let mut code = to_big_endian(c as u32);
            output[output_i] = code[0];
            output[output_i + 1] = code[1];
            output[output_i + 2] = code[2];
            output[output_i + 3] = code[3];
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

    // Loop through the input, getting 4 bytes at a time.
    let mut itr = input.chunks(4);
    while let Some(bytes) = itr.next() {
        if bytes.len() < 4 {
            break;
        }

        // Do the decode.
        if let Some(code) =
            std::char::from_u32(from_big_endian([bytes[0], bytes[1], bytes[2], bytes[3]]))
        {
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
        std::str::from_utf8_unchecked(&output[..output_i])
    }))
}
