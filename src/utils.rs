use ropey::{iter::Chunks, RopeSlice};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};
use unicode_width::UnicodeWidthStr;

pub fn digit_count(mut n: u32, b: u32) -> u32 {
    let mut d = 0;
    loop {
        n /= b;
        d += 1;
        if n == 0 {
            return d;
        }
    }
}

//=============================================================

pub fn grapheme_width(slice: &RopeSlice) -> usize {
    use term_ui::smallstring::SmallString;
    let s = SmallString::from_rope_slice(slice);
    return UnicodeWidthStr::width(&s[..]);
}

/// Finds the previous grapheme boundary before the given char position.
pub fn prev_grapheme_boundary(slice: &RopeSlice, char_idx: usize) -> usize {
    // Bounds check
    debug_assert!(char_idx <= slice.len_chars());

    // We work with bytes for this, so convert.
    let byte_idx = slice.char_to_byte(char_idx);

    // Get the chunk with our byte index in it, and calculate its starting
    // byte within the total rope slice.
    let (mut chunk, byte_offset) = slice.chunk_at_byte(byte_idx);
    let mut chunk_start_idx = byte_idx - byte_offset;

    // Set up the grapheme cursor.
    let mut gc = GraphemeCursor::new(byte_idx, slice.len_bytes(), true);

    // Find the previous grapheme cluster boundary.
    loop {
        match gc.prev_boundary(chunk, chunk_start_idx) {
            Ok(None) => return 0,
            Ok(Some(n)) => return slice.byte_to_char(n),
            Err(GraphemeIncomplete::PrevChunk) => {
                chunk = slice.chunk_at_byte(chunk_start_idx - 1).0;
                chunk_start_idx -= chunk.len();
            }
            Err(GraphemeIncomplete::PreContext(n)) => {
                let (ctx_chunk, _) = slice.chunk_at_byte(n - 1);
                gc.provide_context(ctx_chunk, n - ctx_chunk.len());
            }
            _ => unreachable!(),
        }
    }
}

/// Finds the next grapheme boundary after the given char position.
pub fn next_grapheme_boundary(slice: &RopeSlice, char_idx: usize) -> usize {
    // Bounds check
    debug_assert!(char_idx <= slice.len_chars());

    // We work with bytes for this, so convert.
    let byte_idx = slice.char_to_byte(char_idx);

    // Get the chunk with our byte index in it, and calculate its starting
    // byte within the total rope slice.
    let (mut chunk, byte_offset) = slice.chunk_at_byte(byte_idx);
    let mut chunk_start_idx = byte_idx - byte_offset;

    // Set up the grapheme cursor.
    let mut gc = GraphemeCursor::new(byte_idx, slice.len_bytes(), true);

    // Find the next grapheme cluster boundary.
    loop {
        match gc.next_boundary(chunk, chunk_start_idx) {
            Ok(None) => return slice.len_chars(),
            Ok(Some(n)) => return slice.byte_to_char(n),
            Err(GraphemeIncomplete::NextChunk) => {
                chunk_start_idx += chunk.len();
                chunk = slice.chunk_at_byte(chunk_start_idx).0;
            }
            Err(GraphemeIncomplete::PreContext(n)) => {
                let (ctx_chunk, _) = slice.chunk_at_byte(n - 1);
                gc.provide_context(ctx_chunk, n - ctx_chunk.len());
            }
            _ => unreachable!(),
        }
    }
}

/// Returns whether the given char position is a grapheme boundary.
pub fn is_grapheme_boundary(slice: &RopeSlice, char_idx: usize) -> bool {
    // Bounds check
    debug_assert!(char_idx <= slice.len_chars());

    // We work with bytes for this, so convert.
    let byte_idx = slice.char_to_byte(char_idx);

    // Get the chunk with our byte index in it, and calculate its starting
    // byte within the total rope slice.
    let (chunk, byte_offset) = slice.chunk_at_byte(byte_idx);
    let chunk_start_idx = byte_idx - byte_offset;

    // Set up the grapheme cursor.
    let mut gc = GraphemeCursor::new(byte_idx, slice.len_bytes(), true);

    // Determine if the given position is a grapheme cluster boundary.
    loop {
        match gc.is_boundary(chunk, chunk_start_idx) {
            Ok(n) => return n,
            Err(GraphemeIncomplete::PreContext(n)) => {
                let (ctx_chunk, _) = slice.chunk_at_byte(n - 1);
                gc.provide_context(ctx_chunk, n - ctx_chunk.len());
            }
            _ => unreachable!(),
        }
    }
}

/// An iterator over the graphemes of a RopeSlice.
pub struct RopeGraphemes<'a> {
    text: RopeSlice<'a>,
    chunks: Chunks<'a>,
    cur_chunk: &'a str,
    cur_chunk_start: usize,
    cursor: GraphemeCursor,
}

impl<'a> RopeGraphemes<'a> {
    pub fn new<'b>(slice: &RopeSlice<'b>) -> RopeGraphemes<'b> {
        let mut chunks = slice.chunks();
        let first_chunk = chunks.next().unwrap_or("");
        RopeGraphemes {
            text: *slice,
            chunks: chunks,
            cur_chunk: first_chunk,
            cur_chunk_start: 0,
            cursor: GraphemeCursor::new(0, slice.len_bytes(), true),
        }
    }
}

impl<'a> Iterator for RopeGraphemes<'a> {
    type Item = RopeSlice<'a>;

    fn next(&mut self) -> Option<RopeSlice<'a>> {
        let a = self.cursor.cur_cursor();
        let b;
        loop {
            match self.cursor
                .next_boundary(self.cur_chunk, self.cur_chunk_start)
            {
                Ok(None) => {
                    return None;
                }
                Ok(Some(n)) => {
                    b = n;
                    break;
                }
                Err(GraphemeIncomplete::NextChunk) => {
                    self.cur_chunk_start += self.cur_chunk.len();
                    self.cur_chunk = self.chunks.next().unwrap_or("");
                }
                _ => unreachable!(),
            }
        }

        if a < self.cur_chunk_start {
            let a_char = self.text.byte_to_char(a);
            let b_char = self.text.byte_to_char(b);

            Some(self.text.slice(a_char..b_char))
        } else {
            let a2 = a - self.cur_chunk_start;
            let b2 = b - self.cur_chunk_start;
            Some(RopeSlice::from_str(&self.cur_chunk[a2..b2]))
        }
    }
}

//=============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_count_base_10() {
        assert_eq!(digit_count(0, 10), 1);
        assert_eq!(digit_count(9, 10), 1);
        assert_eq!(digit_count(10, 10), 2);
        assert_eq!(digit_count(99, 10), 2);
        assert_eq!(digit_count(100, 10), 3);
        assert_eq!(digit_count(999, 10), 3);
        assert_eq!(digit_count(1000, 10), 4);
        assert_eq!(digit_count(9999, 10), 4);
        assert_eq!(digit_count(10000, 10), 5);
        assert_eq!(digit_count(99999, 10), 5);
        assert_eq!(digit_count(100000, 10), 6);
        assert_eq!(digit_count(999999, 10), 6);
        assert_eq!(digit_count(1000000, 10), 7);
        assert_eq!(digit_count(9999999, 10), 7);
    }
}
