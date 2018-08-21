#[macro_use]
extern crate proptest;
extern crate text_encoding;

use proptest::collection::vec;
use proptest::test_runner::Config;
use text_encoding::{decode_to_utf8, encode_from_utf8, Encoding};

proptest! {
    #![proptest_config(Config::with_cases(512))]

    #[test]
    fn pt_utf8_roundtrip(ref text in "\\PC*\\PC*\\PC*") {
        let mut buf = [0u8; 32];
        let mut utf8_encoded: Vec<u8> = Vec::new();
        let mut utf8 = String::new();

        // Encode to utf8
        let mut tmp = &text[..];
        while !tmp.is_empty() {
            if let Ok((n, encoded)) = encode_from_utf8(Encoding::Utf8, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf8_encoded.extend_from_slice(encoded);
            } else {
                panic!("Error when encoding.");
            }
        }

        // Decode back from utf8
        let mut tmp = &utf8_encoded[..];
        while !tmp.is_empty() {
            if let Ok((n, decoded)) = decode_to_utf8(Encoding::Utf8, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf8.extend(decoded.chars());
            } else {
                panic!("Error when decoding.");
            }
        }

        assert_eq!(&text[..], &utf8[..]);
        assert_eq!(text.as_bytes(), &utf8_encoded[..]);
        assert_eq!(utf8.as_bytes(), &utf8_encoded[..]);
    }

    #[test]
    fn pt_utf16be_roundtrip(ref text in "\\PC*\\PC*\\PC*") {
        let mut buf = [0u8; 32];
        let mut utf16: Vec<u8> = Vec::new();
        let mut utf8 = String::new();

        // Encode to utf16 big endian
        let mut tmp = &text[..];
        while !tmp.is_empty() {
            if let Ok((n, encoded)) = encode_from_utf8(Encoding::Utf16BE, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf16.extend_from_slice(encoded);
            } else {
                panic!("Error when encoding.");
            }
        }

        // Decode back from utf16 big endian
        let mut tmp = &utf16[..];
        while !tmp.is_empty() {
            if let Ok((n, decoded)) = decode_to_utf8(Encoding::Utf16BE, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf8.extend(decoded.chars());
            } else {
                panic!("Error when decoding.");
            }
        }

        assert_eq!(&text[..], &utf8[..]);
    }

    #[test]
    fn pt_utf16le_roundtrip(ref text in "\\PC*\\PC*\\PC*") {
        let mut buf = [0u8; 32];
        let mut utf16: Vec<u8> = Vec::new();
        let mut utf8 = String::new();

        // Encode to utf16 little endian
        let mut tmp = &text[..];
        while !tmp.is_empty() {
            if let Ok((n, encoded)) = encode_from_utf8(Encoding::Utf16LE, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf16.extend_from_slice(encoded);
            } else {
                panic!("Error when encoding.");
            }
        }

        // Decode back from utf16 big endian
        let mut tmp = &utf16[..];
        while !tmp.is_empty() {
            if let Ok((n, decoded)) = decode_to_utf8(Encoding::Utf16LE, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf8.extend(decoded.chars());
            } else {
                panic!("Error when decoding.");
            }
        }

        assert_eq!(&text[..], &utf8[..]);
    }

    #[test]
    fn pt_utf32be_roundtrip(ref text in "\\PC*\\PC*\\PC*") {
        let mut buf = [0u8; 32];
        let mut utf32: Vec<u8> = Vec::new();
        let mut utf8 = String::new();

        // Encode to utf32 big endian
        let mut tmp = &text[..];
        while !tmp.is_empty() {
            if let Ok((n, encoded)) = encode_from_utf8(Encoding::Utf32BE, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf32.extend_from_slice(encoded);
            } else {
                panic!("Error when encoding.");
            }
        }

        // Decode back from utf32 big endian
        let mut tmp = &utf32[..];
        while !tmp.is_empty() {
            if let Ok((n, decoded)) = decode_to_utf8(Encoding::Utf32BE, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf8.extend(decoded.chars());
            } else {
                panic!("Error when decoding.");
            }
        }

        assert_eq!(&text[..], &utf8[..]);
    }

    #[test]
    fn pt_utf32le_roundtrip(ref text in "\\PC*\\PC*\\PC*") {
        let mut buf = [0u8; 32];
        let mut utf32: Vec<u8> = Vec::new();
        let mut utf8 = String::new();

        // Encode to utf32 little endian
        let mut tmp = &text[..];
        while !tmp.is_empty() {
            if let Ok((n, encoded)) = encode_from_utf8(Encoding::Utf32LE, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf32.extend_from_slice(encoded);
            } else {
                panic!("Error when encoding.");
            }
        }

        // Decode back from utf32 little endian
        let mut tmp = &utf32[..];
        while !tmp.is_empty() {
            if let Ok((n, decoded)) = decode_to_utf8(Encoding::Utf32LE, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf8.extend(decoded.chars());
            } else {
                panic!("Error when decoding.");
            }
        }

        assert_eq!(&text[..], &utf8[..]);
    }

    #[test]
    fn pt_latin1_roundtrip(ref data in vec(0u8..=255, 0..1000)) {
        let mut buf = [0u8; 32];
        let mut utf8 = String::new();
        let mut latin1: Vec<u8> = Vec::new();

        // Decode from latin1 to utf8
        let mut tmp = &data[..];
        while !tmp.is_empty() {
            if let Ok((n, decoded)) = decode_to_utf8(Encoding::Latin1, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf8.extend(decoded.chars());
            } else {
                panic!("Error when decoding.");
            }
        }

        // Encode to from utf8 back to latin1
        let mut tmp = &utf8[..];
        while !tmp.is_empty() {
            if let Ok((n, encoded)) = encode_from_utf8(Encoding::Latin1, tmp, &mut buf) {
                tmp = &tmp[n..];
                latin1.extend_from_slice(encoded);
            } else {
                panic!("Error when encoding.");
            }
        }

        assert_eq!(&data[..], &latin1[..]);
    }

    #[test]
    fn pt_windows1252_roundtrip(mut data in vec(0u8..=255, 0..1000)) {
        let mut buf = [0u8; 32];
        let mut utf8 = String::new();
        let mut w1252: Vec<u8> = Vec::new();

        // Eliminate undefined bytes in input.
        for b in data.iter_mut() {
            if *b == 0x81 || *b == 0x8D || *b == 0x8F || *b == 0x90 || *b == 0x9D {
                *b = 0;
            }
        }

        // Decode from windows-1252 to utf8
        let mut tmp = &data[..];
        while !tmp.is_empty() {
            if let Ok((n, decoded)) = decode_to_utf8(Encoding::Windows1252, tmp, &mut buf) {
                tmp = &tmp[n..];
                utf8.extend(decoded.chars());
            } else {
                panic!("Error when decoding.");
            }
        }

        // Encode to from utf8 back to w1252
        let mut tmp = &utf8[..];
        while !tmp.is_empty() {
            if let Ok((n, encoded)) = encode_from_utf8(Encoding::Windows1252, tmp, &mut buf) {
                tmp = &tmp[n..];
                w1252.extend_from_slice(encoded);
            } else {
                panic!("Error when encoding.");
            }
        }

        assert_eq!(&data[..], &w1252[..]);
    }
}
