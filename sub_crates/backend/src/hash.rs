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
                let (a, b, c) = unsafe { self.buf.align_to::<u64>() };
                debug_assert!(a.is_empty());
                debug_assert!(c.is_empty());
                mix(&mut self.state[..], b);
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
            // Pad with zero.
            for i in (&mut self.buf[self.buf_length..]).iter_mut() {
                *i = 0;
            }

            // Process.
            let (a, b, c) = unsafe { self.buf.align_to::<u64>() };
            debug_assert!(a.is_empty());
            debug_assert!(c.is_empty());
            mix(&mut self.state[..], b);
            self.buf_length = 0;
        }

        // Hash the message length, in bits.
        mix(&mut self.state[..], &[self.message_length * 8, 0, 0, 0]);

        // Convert to little endian.
        self.state[0] = self.state[0].to_le();
        self.state[1] = self.state[1].to_le();
        self.state[2] = self.state[2].to_le();
        self.state[3] = self.state[3].to_le();

        // Return the result.
        unsafe { std::mem::transmute(self.state) }
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

    state.swap(1, 3);
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
    fn hash_string_01() {
        let s = "";
        let correct_digest = "4c0995f905f4e502431432b0e88cacdb4de79d68a61d2ad6606dfbaadac265ac";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_02() {
        let s = "a";
        let correct_digest = "53278d6f86ce2f4948b908b18db874966c755abb662ca5ef71598bebb615b4b7";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_03() {
        let s = "aaa";
        let correct_digest = "57026c6accb4eb5cfd2423481945f752705085407dddfaf170537943aa3b184d";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_04() {
        let s = "abc";
        let correct_digest = "924ab7415342ded68e5e16d9a0bac734b9a0652b81aaef2566415f3ef95294c1";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_05() {
        let s = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let correct_digest = "5a277cb9e6085ee07b22ccc6159d67678643fb85f83997b58d2d8e7d8ca176c9";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_06() {
        let s = "The quick brown fox jumps over the lazy dog";
        let correct_digest = "f3eba2aee55207c45839d3e5250943a32cbd85477f383e136d7bfc8e9bf737b6";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_07() {
        let s = "The quick brown fox jumps over the lazy dog.";
        let correct_digest = "b39d81bb4eec906cbaf516c29de4c31069fcc32ffd388d63f683ba323eedd759";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_08() {
        let s = "message digest";
        let correct_digest = "18e021a1b1fb654d1bcf7aa73eeebe0d271f5595a450e8d437c9a609f89f086d";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_09() {
        let s = "abcdefghijklmnopqrstuvwxyz";
        let correct_digest = "1c65e9cdfae2fd10ebd2b7da4dc5b99fad4fc3b9cf76bafdc67a770b5e797c98";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_10() {
        let s = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let correct_digest = "834da12435130ae02ced77541a96815099ebdfc43bfa7ee277c3b4f64c2243cf";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_11() {
        let s = "12345678901234567890123456789012345678901234567890123456789012345678901234567890";
        let correct_digest = "141b3cc6f5cb6e756dcf1a16ebc975ac702994ddee7b1a2293072a3c2c11b51d";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_string_12() {
        let s = "Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        let correct_digest = "5fbe85a0a525f3b648542492751e6ae3c05113a3c46dd48aa690ac720126f476";
        assert_eq!(digest_to_string(hash(s.as_bytes())), correct_digest);
    }

    #[test]
    fn hash_length() {
        // We're testing here to make sure the length of the data properly
        // affects the hash.  The last block of data is padded with zeros
        // if less than the block size, so here we're forcing that last
        // block to be all zeros, and only changing the length of input.
        let len_0 = &[];
        let len_1 = &[0u8];
        let len_2 = &[0u8, 0];

        let len_0_hash = digest_to_string(hash(len_0));
        let len_1_hash = digest_to_string(hash(len_1));
        let len_2_hash = digest_to_string(hash(len_2));

        assert!(len_0_hash != len_1_hash);
        assert!(len_0_hash != len_2_hash);
        assert!(len_1_hash != len_2_hash);
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
        let correct_digest = "5fbe85a0a525f3b648542492751e6ae3c05113a3c46dd48aa690ac720126f476";

        let mut hasher = LedHash256::new();
        hasher.update(test_string1.as_bytes());
        hasher.update(test_string2.as_bytes());
        hasher.update(test_string3.as_bytes());
        hasher.update(test_string4.as_bytes());
        hasher.update(test_string5.as_bytes());
        let digest = hasher.finish();

        assert_eq!(digest_to_string(digest), correct_digest);
    }
}
