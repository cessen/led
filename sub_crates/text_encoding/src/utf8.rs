//! These functions are essentially redundant, since they're supposedly
//! encoding/decoding between utf8 and... utf8.  However, `decode_to_utf8()`
//! is still useful for validating unknown input.  And they allow a uniform
//! API for all encodings.

use std;
use {DecodeError, DecodeResult, EncodeResult};

// Encode from utf8
pub fn encode_from_utf8<'a>(input: &str, output: &'a mut [u8]) -> EncodeResult<'a> {
    let copy_len = {
        if output.len() >= input.len() {
            input.len()
        } else {
            let mut i = output.len();
            while !input.is_char_boundary(i) {
                i -= 1;
            }
            i
        }
    };

    output[..copy_len].copy_from_slice(input[..copy_len].as_bytes());

    Ok((copy_len, &output[..copy_len]))
}

pub fn decode_to_utf8<'a>(input: &[u8], output: &'a mut [u8]) -> DecodeResult<'a> {
    let valid_up_to = match std::str::from_utf8(input) {
        Ok(text) => text.len(),
        Err(e) => {
            if e.valid_up_to() > 0 {
                e.valid_up_to()
            } else {
                return Err(DecodeError {
                    error_range: (0, 1), // TODO: search for the next starting byte to get the range.
                    output_bytes_written: 0,
                });
            }
        }
    };

    let (in_consumed, out_slice) = encode_from_utf8(
        unsafe { std::str::from_utf8_unchecked(&input[..valid_up_to]) },
        output,
    ).unwrap();

    Ok((in_consumed, unsafe {
        std::str::from_utf8_unchecked(out_slice)
    }))
}
