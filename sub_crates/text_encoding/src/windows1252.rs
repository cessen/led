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
            input_i = offset;
        } else {
            return Err(EncodeError {
                character: c,
                error_range: (offset, offset + c.len_utf8()),
                output_bytes_written: output_i,
            });
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
        '€' => Some(0x80),
        '‚' => Some(0x82),
        'ƒ' => Some(0x83),
        '„' => Some(0x84),
        '…' => Some(0x85),
        '†' => Some(0x86),
        '‡' => Some(0x87),
        'ˆ' => Some(0x88),
        '‰' => Some(0x89),
        'Š' => Some(0x8A),
        '‹' => Some(0x8B),
        'Œ' => Some(0x8C),
        'Ž' => Some(0x8E),
        '‘' => Some(0x91),
        '’' => Some(0x92),
        '“' => Some(0x93),
        '”' => Some(0x94),
        '•' => Some(0x95),
        '–' => Some(0x96),
        '—' => Some(0x97),
        '˜' => Some(0x98),
        '™' => Some(0x99),
        'š' => Some(0x9A),
        '›' => Some(0x9B),
        'œ' => Some(0x9C),
        'ž' => Some(0x9E),
        'Ÿ' => Some(0x9F),
        _ => None,
    }
}

// Maps the range 0x80-0x9F in windows-1252 to unicode.  The remaining
// characters in windows-1252 match unicode.
//
// The '�'s stand in for codes not defined in windows-1252, and should be
// be treated as an error when encountered.
const DECODE_TABLE: [char; 32] = [
    '€', '�', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '�',
    'Ž', '�', '�', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›',
    'œ', '�', 'ž', 'Ÿ',
];
