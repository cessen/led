/// A 256-bit non-cryptographic hash function for data identification.
///
/// This uses the MIX function, constants, and permutation patterns
/// from Skein v1.3, but is otherwise largely unrelated--in particular
/// it does not use sub-keys, tweak values, or UBI from Skein.
///
/// This implementation assumes support for 64-bit unsigned integers.
///
/// This implementation should work on platforms of any endianness,
/// but has only been tested on little endian platforms.  Running the
/// unit tests on a big-endian platform can verify.

const BLOCK_SIZE: usize = 256 / 8; // Block size of the hash, in bytes

/// Convenience function to generate a hash for a block of data.
pub fn hash(data: &[u8]) -> [u8; BLOCK_SIZE] {
    let mut h = LedHash256::new();
    h.update(data);
    h.finish()
}

/// A hash builder.  Consumes bytes and generates a 256-bit hash.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
#[repr(align(8))]
pub struct LedHash256 {
    state: [u64; 4],       // Hash state.
    buf: [u8; BLOCK_SIZE], // Accumulates message data for processing.
    buf_length: usize,     // The number of message bytes currently stored in buf[].
    message_length: u64,   // Accumulates the total message length, in bytes.
}

impl LedHash256 {
    pub fn new() -> LedHash256 {
        LedHash256 {
            state: [
                // Initial Chaining Values from Skein-256-256, v1.3
                0xFC9DA860D048B449,
                0x2FCA66479FA7D833,
                0xB33BC3896656840F,
                0x6A54E920FDE8DA69,
            ],
            buf: [0; BLOCK_SIZE],
            buf_length: 0,
            message_length: 0,
        }
    }

    /// Update the hash with new data.
    pub fn update(&mut self, data: &[u8]) {
        self.message_length += data.len() as u64;

        let mut data = data;

        while !data.is_empty() {
            if self.buf_length >= BLOCK_SIZE {
                // Process the filled buffer
                self.mix_buffer_into_state();
                self.buf_length = 0;
            } else {
                // Fill the buffer.
                let n = (BLOCK_SIZE - self.buf_length).min(data.len());
                (&mut self.buf[self.buf_length..(self.buf_length + n)]).copy_from_slice(&data[..n]);
                data = &data[n..];
                self.buf_length += n;
            }
        }
    }

    /// Finishes the hash calculations and returns the digest.
    pub fn finish(mut self) -> [u8; BLOCK_SIZE] {
        // Hash the remaining bytes if there are any.
        if self.buf_length > 0 {
            for i in (&mut self.buf[self.buf_length..]).iter_mut() {
                *i = 0;
            }
            self.mix_buffer_into_state();
            self.buf_length = 0;
        }

        // Hash the message length, in bits.
        mix(&mut self.state[..], &[self.message_length * 8, 0, 0, 0]);

        // Get the digest as a byte array and return it.
        let mut result = [0u8; BLOCK_SIZE];
        result[0..8].copy_from_slice(&self.state[0].to_le_bytes());
        result[8..16].copy_from_slice(&self.state[1].to_le_bytes());
        result[16..24].copy_from_slice(&self.state[2].to_le_bytes());
        result[24..32].copy_from_slice(&self.state[3].to_le_bytes());
        return result;
    }

    fn mix_buffer_into_state(&mut self) {
        let (a, b, c) = unsafe { self.buf.align_to::<u64>() };
        debug_assert!(a.is_empty());
        debug_assert!(c.is_empty());
        mix(&mut self.state[..], b);
    }
}

/// The main mix function.  Mixes a block into the hash state.
///
/// Inspired by Skein 1.3, and using the constants from its 256-bit
/// variant.  It does 9 rounds of mixing, as that produces full
/// diffusion for 256-bit keys according to the Skein 1.3 paper.
///
/// The mix rotation constants, as taken from Skein 1.3 256-bit variant:
/// 14 16
/// 52 57
/// 23 40
///  5 37
/// 25 33
/// 46 12
/// 58 22
/// 32 32
/// repeat
///
/// The permute table, as taken from Skein 1.3 256-bit variant:
/// Indices: 0 1 2 3
/// Become:  0 3 2 1
fn mix(state: &mut [u64], block: &[u64]) {
    /// The MIX function from Skein.
    fn umix(pair: &mut [u64], r: u32) {
        pair[0] = pair[0].wrapping_add(pair[1]);
        pair[1] = pair[1].rotate_left(r) ^ pair[0];
    }

    // Convert the block to native endianness and xor into the hash state.
    state[0] ^= u64::from_le(block[0]);
    state[1] ^= u64::from_le(block[1]);
    state[2] ^= u64::from_le(block[2]);
    state[3] ^= u64::from_le(block[3]);

    // Mixing constants.
    const ROUNDS: usize = 9;
    const ROTATION_TABLE: [(u32, u32); 8] = [
        (14, 16),
        (52, 57),
        (23, 40),
        (5, 37),
        (25, 33),
        (46, 12),
        (58, 22),
        (32, 32),
    ];

    // Do the mixing.
    for (rot_1, rot_2) in ROTATION_TABLE.iter().cycle().take(ROUNDS) {
        umix(&mut state[..2], *rot_1);
        umix(&mut state[2..], *rot_2);
        state.swap(1, 3);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn digest_to_string(digest: [u8; 32]) -> String {
        fn low_bits_to_char(n: u8) -> char {
            match n {
                0 => '0',
                1 => '1',
                2 => '2',
                3 => '3',
                4 => '4',
                5 => '5',
                6 => '6',
                7 => '7',
                8 => '8',
                9 => '9',
                10 => 'a',
                11 => 'b',
                12 => 'c',
                13 => 'd',
                14 => 'e',
                15 => 'f',
                _ => unreachable!(),
            }
        }

        let mut s = String::new();
        for byte in &digest {
            s.push(low_bits_to_char(byte >> 4u8));
            s.push(low_bits_to_char(byte & 0b00001111));
        }
        s
    }

    #[test]
    fn hash_empty() {
        let correct_digest = "4c0995f905f4e502606dfbaadac265ac4de79d68a61d2ad6431432b0e88cacdb";
        assert_eq!(digest_to_string(hash(&[])), correct_digest);
    }

    #[test]
    fn hash_zero() {
        let correct_digest = "9061db6180de3b2193eca59ada3d00472d0bb25ecced849b55586b367d8e10bd";
        assert_eq!(digest_to_string(hash(&[0u8])), correct_digest);
    }

    #[test]
    fn hash_one() {
        let correct_digest = "a781e5d5375ece4306b8406e25132be4b50a0af93d544a280dc67cde890235a8";
        assert_eq!(digest_to_string(hash(&[1u8])), correct_digest);
    }

    #[test]
    fn hash_string_01() {
        let s = "abc";
        let correct_digest = "08c83bd40744cd323890d1d1ca72274c4ec4a0e1de0b68761248b1bdd70a845a";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_02() {
        let s = "The quick brown fox jumps over the lazy dog.";
        let correct_digest = "f94bd8c035958bb27c5a733ad7efd21286da12b9cf33a6496d9b9e35813720b7";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_03() {
        let s = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let correct_digest = "9e54edbe1611f9c5e0a306240537df41d2ca1d584a125119cbf09fdb2f6ab880";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_04() {
        let s = "Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        let correct_digest = "5e4a25a709236e55995eee313495ae1d43c09d4acfec69a107b65be20b6c3863";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_multi_part_processing() {
        let test_string1 =
            "Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor";
        let test_string2 = " incididunt ut l";
        let test_string3 = "abore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat ";
        let test_string4 = "cup";
        let test_string5 =
            "idatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        let correct_digest = "5e4a25a709236e55995eee313495ae1d43c09d4acfec69a107b65be20b6c3863";

        let mut hasher = LedHash256::new();
        hasher.update(test_string1.as_bytes());
        hasher.update(test_string2.as_bytes());
        hasher.update(test_string3.as_bytes());
        hasher.update(test_string4.as_bytes());
        hasher.update(test_string5.as_bytes());
        let digest = hasher.finish();

        assert_eq!(digest_to_string(digest), correct_digest);
    }

    #[test]
    fn hash_length() {
        // We're testing here to make sure the length of the data properly
        // affects the hash.  Internally in the hash, the last block of data
        // is padded with zeros, so here we're forcing that last block to be
        // all zeros, and only changing the length of input.
        let len_0 = &[];
        let len_1 = &[0u8];
        let len_2 = &[0u8, 0];

        assert_eq!(
            digest_to_string(hash(len_0)),
            "4c0995f905f4e502606dfbaadac265ac4de79d68a61d2ad6431432b0e88cacdb",
        );
        assert_eq!(
            digest_to_string(hash(len_1)),
            "9061db6180de3b2193eca59ada3d00472d0bb25ecced849b55586b367d8e10bd",
        );
        assert_eq!(
            digest_to_string(hash(len_2)),
            "054c0440b479e206a28923f5924b7bc0d3f0ead1eaff3360aa56750b98fd5645",
        );
    }
}
