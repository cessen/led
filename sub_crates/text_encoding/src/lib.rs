//! A library for incrementally encoding/decoding between utf8 and various
//! text encodings.

mod latin1;
mod utf16_be;
mod utf16_le;
mod utf8;

/// Encodes text from utf8 to a destination encoding.
pub fn encode_from_utf8(output_encoding: Encoding, input: &str, output: &mut [u8]) -> EncodeResult {
    match output_encoding {
        Encoding::Utf8 => utf8::encode_from_utf8(input, output),
        Encoding::Utf16BE => utf16_be::encode_from_utf8(input, output),
        Encoding::Utf16LE => utf16_le::encode_from_utf8(input, output),
        Encoding::Latin1 => latin1::encode_from_utf8(input, output),
        _ => unimplemented!(),
    }
}

/// Decodes text from a source encoding to utf8.
pub fn decode_to_utf8<'a>(
    input_encoding: Encoding,
    input: &[u8],
    output: &'a mut [u8],
) -> DecodeResult<'a> {
    match input_encoding {
        Encoding::Utf8 => utf8::decode_to_utf8(input, output),
        Encoding::Utf16BE => utf16_be::decode_to_utf8(input, output),
        Encoding::Utf16LE => utf16_le::decode_to_utf8(input, output),
        Encoding::Latin1 => latin1::decode_to_utf8(input, output),
        _ => unimplemented!(),
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
    ShiftJIS,
    Big5,
    Latin1,      // ISO/IEC 8859-1
    Windows1252, // Windows code page 1252
}

/// Result type for encoding text from utf8 to a target encoding.
///
/// The Ok() variant provides the number of bytes consumed and the
/// number of bytes written, in that order.
pub type EncodeResult = Result<(usize, usize), EncodeError>;

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
/// The problematic character, the byte offset of that character
/// in the input utf8, and the number of bytes already written to the output
/// buffer is provided.
///
/// It is guaranteed that all input leading up to the problem character has
/// already been encoded and written to the output buffer.
#[derive(Debug, Copy, Clone)]
pub struct EncodeError {
    pub character: char,
    pub byte_offset: usize,
    pub bytes_written: usize,
}

/// Represents an error when decoding to utf8 from some other format.
///
/// All supported text encodings can be fully represented in utf8, and
/// therefore the only possible error is that we encounter bytes in the
/// input data that are invalid for the text encoding we're attempting
/// to decode from.
///
/// The byte offset of the invalid input data and in the number of bytes
/// already written to the output buffer are.
/// already been encoded and written to the output buffer.
///
/// It is guaranteed that all input leading up to the invalid data has
/// already been encoded and written to the output buffer.
#[derive(Debug, Copy, Clone)]
pub struct DecodeError {
    pub byte_offset: usize,
    pub bytes_written: usize,
}
