#![no_std]

//! A library for incrementally encoding/decoding between utf8 and various
//! text encodings.

mod latin1;
mod single_byte;
mod utf16_be;
mod utf16_le;
mod utf32_be;
mod utf32_le;
mod utf8;
mod utils;

use single_byte::{ibm866, iso_8859_2, iso_8859_7, windows1252};

/// Encodes text from utf8 to a destination encoding.
pub fn encode_from_str<'a>(
    output_encoding: Encoding,
    input: &str,
    output: &'a mut [u8],
) -> EncodeResult<'a> {
    match output_encoding {
        Encoding::Utf8 => utf8::encode_from_str(input, output),
        Encoding::Utf16BE => utf16_be::encode_from_str(input, output),
        Encoding::Utf16LE => utf16_le::encode_from_str(input, output),
        Encoding::Utf32BE => utf32_be::encode_from_str(input, output),
        Encoding::Utf32LE => utf32_le::encode_from_str(input, output),
        Encoding::IBM866 => ibm866::encode_from_str(input, output),
        Encoding::Latin1 => latin1::encode_from_str(input, output),
        Encoding::ISO8859_2 => iso_8859_2::encode_from_str(input, output),
        Encoding::ISO8859_7 => iso_8859_7::encode_from_str(input, output),
        Encoding::Windows1252 => windows1252::encode_from_str(input, output),
    }
}

/// Decodes text from a source encoding to utf8.
pub fn decode_to_str<'a>(
    input_encoding: Encoding,
    input: &[u8],
    output: &'a mut [u8],
) -> DecodeResult<'a> {
    match input_encoding {
        Encoding::Utf8 => utf8::decode_to_str(input, output),
        Encoding::Utf16BE => utf16_be::decode_to_str(input, output),
        Encoding::Utf16LE => utf16_le::decode_to_str(input, output),
        Encoding::Utf32BE => utf32_be::decode_to_str(input, output),
        Encoding::Utf32LE => utf32_le::decode_to_str(input, output),
        Encoding::IBM866 => ibm866::decode_to_str(input, output),
        Encoding::Latin1 => latin1::decode_to_str(input, output),
        Encoding::ISO8859_2 => iso_8859_2::decode_to_str(input, output),
        Encoding::ISO8859_7 => iso_8859_7::decode_to_str(input, output),
        Encoding::Windows1252 => windows1252::decode_to_str(input, output),
    }
}

/// Describes a text encoding.
#[derive(Debug, Copy, Clone)]
pub enum Encoding {
    Utf8,
    Utf16BE, // Big endian
    Utf16LE, // Little endian
    Utf32BE, // Big endian
    Utf32LE, // Little endian
    // ShiftJIS,
    // EUC_JP,
    // Big5,
    IBM866,      // IBM 866
    Latin1,      // ISO/IEC 8859-1
    ISO8859_2,   // ISO/IEC 8859-2
    ISO8859_7,   // ISO/IEC 8859-7
    Windows1252, // Windows code page 1252
}

/// Result type for encoding text from utf8 to a target encoding.
///
/// The Ok() variant provides the number of bytes consumed and a reference
/// to the valid encoded text data.
pub type EncodeResult<'a> = Result<(usize, &'a [u8]), EncodeError>;

/// Result type for decoding text from a target encoding to utf8.
///
/// The Ok() variant provides the number of bytes consumed and a reference
/// to the valid decoded text.
pub type DecodeResult<'a> = Result<(usize, &'a str), DecodeError>;

/// Represents an error when encoding from utf8 to some other format.
///
/// Since valid input utf8 is statically assumed, the only possible
/// error is encountering a char that is not representable in the target
/// encoding.
///
/// The problematic character, the byte index range of that character in the
/// input utf8, and the number of bytes already written to the output buffer
/// are provided.
///
/// It is guaranteed that all input leading up to the problem character has
/// already been encoded and written to the output buffer.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EncodeError {
    pub character: char,
    pub error_range: (usize, usize),
    pub output_bytes_written: usize,
}

/// Represents an error when decoding to utf8 from some other format.
///
/// All supported text encodings can be fully represented in utf8, and
/// therefore the only possible error is that we encounter bytes in the
/// input data that are invalid for the text encoding we're attempting
/// to decode from.
///
/// The byte index range of the invalid input data and the number of bytes
/// already encoded and written to the output buffer are provided.
///
/// It is guaranteed that all input leading up to the invalid data has
/// already been encoded and written to the output buffer.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DecodeError {
    pub error_range: (usize, usize),
    pub output_bytes_written: usize,
}
