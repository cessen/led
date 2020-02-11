use std::{borrow::Cow, cmp::max};

use crate::{
    formatter::{LineFormatter, RoundingBehavior},
    string_utils::{is_line_ending, is_whitespace},
    utils::{grapheme_width, RopeGraphemes},
};

pub enum WrapType {
    NoWrap,
    CharWrap(usize),
    WordWrap(usize),
}

// ===================================================================
// LineFormatter implementation for terminals/consoles.
// ===================================================================

pub struct ConsoleLineFormatter {
    pub tab_width: u8,
    pub wrap_type: WrapType,
    pub maintain_indent: bool,
    pub wrap_additional_indent: usize,
}

impl ConsoleLineFormatter {
    pub fn new(tab_width: u8) -> ConsoleLineFormatter {
        ConsoleLineFormatter {
            tab_width: tab_width,
            wrap_type: WrapType::WordWrap(40),
            maintain_indent: true,
            wrap_additional_indent: 0,
        }
    }

    pub fn set_wrap_width(&mut self, width: usize) {
        match self.wrap_type {
            WrapType::NoWrap => {}

            WrapType::CharWrap(ref mut w) => {
                *w = width;
            }

            WrapType::WordWrap(ref mut w) => {
                *w = width;
            }
        }
    }

    pub fn iter<'a>(&self, g_iter: RopeGraphemes<'a>) -> FormattingIter<'a> {
        FormattingIter {
            grapheme_itr: g_iter,
            wrap_width: match self.wrap_type {
                WrapType::WordWrap(w) => w,
                WrapType::CharWrap(w) => w,
                WrapType::NoWrap => unreachable!(),
            },
            tab_width: self.tab_width as usize,
            word_buf: Vec::new(),
            word_i: 0,
            pos: (0, 0),
        }
    }
}

impl LineFormatter for ConsoleLineFormatter {
    fn dimensions(&self, g_iter: RopeGraphemes) -> (usize, usize) {
        let mut dim: (usize, usize) = (0, 0);

        for (_, pos, width) in self.iter(g_iter) {
            dim = (max(dim.0, pos.0), max(dim.1, pos.1 + width));
        }

        dim.0 += 1;

        return dim;
    }

    fn index_to_v2d(&self, g_iter: RopeGraphemes, char_idx: usize) -> (usize, usize) {
        let mut pos = (0, 0);
        let mut i = 0;
        let mut last_width = 0;

        for (g, _pos, width) in self.iter(g_iter) {
            pos = _pos;
            last_width = width;
            i += g.chars().count();

            if i > char_idx {
                return pos;
            }
        }

        return (pos.0, pos.1 + last_width);
    }

    fn v2d_to_index(
        &self,
        g_iter: RopeGraphemes,
        v2d: (usize, usize),
        _: (RoundingBehavior, RoundingBehavior),
    ) -> usize {
        // TODO: handle rounding modes
        let mut prev_i = 0;
        let mut i = 0;

        for (g, pos, _) in self.iter(g_iter) {
            if pos.0 > v2d.0 {
                i = prev_i;
                break;
            } else if pos.0 == v2d.0 && pos.1 >= v2d.1 {
                break;
            }

            prev_i = i;
            i += g.chars().count();
        }

        return i;
    }
}

//--------------------------------------------------------------------------

/// An iterator over the visual printable characters of a piece of text,
/// yielding the text of the character, its position in 2d space, and its
/// visial width.
///
/// TODO: handle maintaining indent, etc.
pub struct FormattingIter<'a> {
    grapheme_itr: RopeGraphemes<'a>,
    wrap_width: usize,
    tab_width: usize,

    word_buf: Vec<(Cow<'a, str>, usize)>, // Printable character and its width.
    word_i: usize,

    pos: (usize, usize),
}

impl<'a> Iterator for FormattingIter<'a> {
    type Item = (Cow<'a, str>, (usize, usize), usize);

    fn next(&mut self) -> Option<Self::Item> {
        // Get next word if necessary
        if self.word_i >= self.word_buf.len() {
            let mut word_width = 0;
            self.word_buf.truncate(0);

            while let Some(g) = self.grapheme_itr.next().map(|g| Cow::<str>::from(g)) {
                let width =
                    grapheme_vis_width_at_vis_pos(&g, self.pos.1 + word_width, self.tab_width);
                self.word_buf.push((g.clone(), width));
                word_width += width;

                if is_whitespace(&g) {
                    break;
                }
            }

            if self.word_buf.len() == 0 {
                return None;
            }

            // Move to next line if necessary
            if (self.pos.1 + word_width) > self.wrap_width {
                if self.pos.1 > 0 {
                    self.pos = (self.pos.0 + 1, 0);
                }
            }

            self.word_i = 0;
        }

        // Get next grapheme and width from the current word.
        let (g, g_width) = {
            let (ref g, mut width) = self.word_buf[self.word_i];
            if g == "\t" {
                width = grapheme_vis_width_at_vis_pos(&g, self.pos.1, self.tab_width);
            }
            (g, width)
        };

        // Get our character's position and update the position for the next
        // grapheme.
        if (self.pos.1 + g_width) > self.wrap_width && self.pos.1 > 0 {
            self.pos.0 += 1;
            self.pos.1 = 0;
        }
        let pos = self.pos;
        self.pos.1 += g_width;

        // Increment index and return.
        self.word_i += 1;
        return Some((g.clone(), pos, g_width));
    }
}

/// Returns the visual width of a grapheme given a starting
/// position on a line.
fn grapheme_vis_width_at_vis_pos(g: &str, pos: usize, tab_width: usize) -> usize {
    if g == "\t" {
        let ending_pos = ((pos / tab_width) + 1) * tab_width;
        return ending_pos - pos;
    } else if is_line_ending(g) {
        return 1;
    } else {
        return grapheme_width(&g);
    }
}

//--------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use ropey::Rope;

    use crate::buffer::Buffer;
    use crate::formatter::LineFormatter;
    use crate::formatter::RoundingBehavior::{Ceiling, Floor, Round};
    use crate::utils::RopeGraphemes;

    use super::*;

    #[test]
    fn dimensions_1() {
        let text = Rope::from_str("Hello there, stranger!"); // 22 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(80);

        assert_eq!(f.dimensions(RopeGraphemes::new(&text.slice(..))), (1, 22));
    }

    #[test]
    fn dimensions_2() {
        let text = Rope::from_str("Hello there, stranger!  How are you doing this fine day?"); // 56 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(12);

        assert_eq!(f.dimensions(RopeGraphemes::new(&text.slice(..))), (5, 12));
    }

    #[test]
    fn dimensions_3() {
        let text = Rope::from_str("Hello there, stranger!  How are you doing this fine day?"); // 56 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::WordWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(12);

        assert_eq!(f.dimensions(RopeGraphemes::new(&text.slice(..))), (6, 12));
    }

    #[test]
    fn dimensions_4() {
        // 55 graphemes long
        let text = Rope::from_str(
            "税マイミ文末\
             レ日題イぽじ\
             や男目統ス公\
             身みトしつ結\
             煮ヱマレ断西\
             ロ領視りいぽ\
             凱字テ式重反\
             てす献罪がご\
             く官俵呉嫁ー\
             。",
        );

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(12);

        assert_eq!(f.dimensions(RopeGraphemes::new(&text.slice(..))), (10, 12));
    }

    #[test]
    fn dimensions_5() {
        // 55 graphemes long
        let text = Rope::from_str(
            "税マイミ文末\
             レ日題イぽじ\
             や男目統ス公\
             身みトしつ結\
             煮ヱマレ断西\
             ロ領視りいぽ\
             凱字テ式重反\
             てす献罪がご\
             く官俵呉嫁ー\
             。",
        );

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::WordWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(12);

        assert_eq!(f.dimensions(RopeGraphemes::new(&text.slice(..))), (10, 12));
    }

    #[test]
    fn index_to_v2d_1() {
        let text = Rope::from_str("Hello there, stranger!"); // 22 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(80);

        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 0),
            (0, 0)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 5),
            (0, 5)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 22),
            (0, 22)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 23),
            (0, 22)
        );
    }

    #[test]
    fn index_to_v2d_2() {
        let text = Rope::from_str("Hello there, stranger!  How are you doing this fine day?"); // 56 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(12);

        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 0),
            (0, 0)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 5),
            (0, 5)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 11),
            (0, 11)
        );

        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 12),
            (1, 0)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 15),
            (1, 3)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 23),
            (1, 11)
        );

        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 24),
            (2, 0)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 28),
            (2, 4)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 35),
            (2, 11)
        );

        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 36),
            (3, 0)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 43),
            (3, 7)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 47),
            (3, 11)
        );

        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 48),
            (4, 0)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 50),
            (4, 2)
        );
        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 56),
            (4, 8)
        );

        assert_eq!(
            f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 57),
            (4, 8)
        );
    }

    #[test]
    fn v2d_to_index_1() {
        let text = Rope::from_str("Hello there, stranger!"); // 22 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(80);

        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 0), (Floor, Floor)),
            0
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 5), (Floor, Floor)),
            5
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 22), (Floor, Floor)),
            22
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 23), (Floor, Floor)),
            22
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (1, 0), (Floor, Floor)),
            22
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (1, 1), (Floor, Floor)),
            22
        );
    }

    #[test]
    fn v2d_to_index_2() {
        let text = Rope::from_str("Hello there, stranger!  How are you doing this fine day?"); // 56 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(12);

        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 0), (Floor, Floor)),
            0
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 11), (Floor, Floor)),
            11
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 12), (Floor, Floor)),
            11
        );

        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (1, 0), (Floor, Floor)),
            12
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (1, 11), (Floor, Floor)),
            23
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (1, 12), (Floor, Floor)),
            23
        );

        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (2, 0), (Floor, Floor)),
            24
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (2, 11), (Floor, Floor)),
            35
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (2, 12), (Floor, Floor)),
            35
        );

        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (3, 0), (Floor, Floor)),
            36
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (3, 11), (Floor, Floor)),
            47
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (3, 12), (Floor, Floor)),
            47
        );

        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (4, 0), (Floor, Floor)),
            48
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (4, 7), (Floor, Floor)),
            55
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (4, 8), (Floor, Floor)),
            56
        );
        assert_eq!(
            f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (4, 9), (Floor, Floor)),
            56
        );
    }

    #[test]
    fn index_to_horizontal_v2d_1() {
        let b = Buffer::new_from_str("Hello there, stranger!\nHow are you doing this fine day?"); // 55 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(80);

        assert_eq!(f.index_to_horizontal_v2d(&b, 0), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 5), 5);
        assert_eq!(f.index_to_horizontal_v2d(&b, 26), 3);
        assert_eq!(f.index_to_horizontal_v2d(&b, 55), 32);
        assert_eq!(f.index_to_horizontal_v2d(&b, 56), 32);
    }

    #[test]
    fn index_to_horizontal_v2d_2() {
        let b = Buffer::new_from_str("Hello there, stranger!\nHow are you doing this fine day?"); // 55 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(12);

        assert_eq!(f.index_to_horizontal_v2d(&b, 0), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 11), 11);

        assert_eq!(f.index_to_horizontal_v2d(&b, 12), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 22), 10);

        assert_eq!(f.index_to_horizontal_v2d(&b, 23), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 34), 11);

        assert_eq!(f.index_to_horizontal_v2d(&b, 35), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 46), 11);

        assert_eq!(f.index_to_horizontal_v2d(&b, 47), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 55), 8);
        assert_eq!(f.index_to_horizontal_v2d(&b, 56), 8);
    }

    #[test]
    fn index_to_horizontal_v2d_3() {
        let b = Buffer::new_from_str("Hello there, stranger!\nHow are you doing this fine day?"); // 55 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::WordWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(12);

        assert_eq!(f.index_to_horizontal_v2d(&b, 0), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 5), 5);

        assert_eq!(f.index_to_horizontal_v2d(&b, 6), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 12), 6);

        assert_eq!(f.index_to_horizontal_v2d(&b, 13), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 22), 9);

        assert_eq!(f.index_to_horizontal_v2d(&b, 23), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 34), 11);

        assert_eq!(f.index_to_horizontal_v2d(&b, 35), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 45), 10);

        assert_eq!(f.index_to_horizontal_v2d(&b, 46), 0);
        assert_eq!(f.index_to_horizontal_v2d(&b, 55), 9);
        assert_eq!(f.index_to_horizontal_v2d(&b, 56), 9);
    }

    #[test]
    fn index_set_horizontal_v2d_1() {
        let b = Buffer::new_from_str("Hello there, stranger!\nHow are you doing this fine day?"); // 55 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(80);

        assert_eq!(f.index_set_horizontal_v2d(&b, 0, 0, Floor), 0);
        assert_eq!(f.index_set_horizontal_v2d(&b, 0, 22, Floor), 22);
        assert_eq!(f.index_set_horizontal_v2d(&b, 0, 23, Floor), 22);

        assert_eq!(f.index_set_horizontal_v2d(&b, 8, 0, Floor), 0);
        assert_eq!(f.index_set_horizontal_v2d(&b, 8, 22, Floor), 22);
        assert_eq!(f.index_set_horizontal_v2d(&b, 8, 23, Floor), 22);

        assert_eq!(f.index_set_horizontal_v2d(&b, 22, 0, Floor), 0);
        assert_eq!(f.index_set_horizontal_v2d(&b, 22, 22, Floor), 22);
        assert_eq!(f.index_set_horizontal_v2d(&b, 22, 23, Floor), 22);

        assert_eq!(f.index_set_horizontal_v2d(&b, 23, 0, Floor), 23);
        assert_eq!(f.index_set_horizontal_v2d(&b, 23, 32, Floor), 55);
        assert_eq!(f.index_set_horizontal_v2d(&b, 23, 33, Floor), 55);

        assert_eq!(f.index_set_horizontal_v2d(&b, 28, 0, Floor), 23);
        assert_eq!(f.index_set_horizontal_v2d(&b, 28, 32, Floor), 55);
        assert_eq!(f.index_set_horizontal_v2d(&b, 28, 33, Floor), 55);

        assert_eq!(f.index_set_horizontal_v2d(&b, 55, 0, Floor), 23);
        assert_eq!(f.index_set_horizontal_v2d(&b, 55, 32, Floor), 55);
        assert_eq!(f.index_set_horizontal_v2d(&b, 55, 33, Floor), 55);
    }

    #[test]
    fn index_set_horizontal_v2d_2() {
        let b = Buffer::new_from_str("Hello there, stranger! How are you doing this fine day?"); // 55 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(12);

        assert_eq!(f.index_set_horizontal_v2d(&b, 0, 0, Floor), 0);
        assert_eq!(f.index_set_horizontal_v2d(&b, 0, 11, Floor), 11);
        assert_eq!(f.index_set_horizontal_v2d(&b, 0, 12, Floor), 11);

        assert_eq!(f.index_set_horizontal_v2d(&b, 8, 0, Floor), 0);
        assert_eq!(f.index_set_horizontal_v2d(&b, 8, 11, Floor), 11);
        assert_eq!(f.index_set_horizontal_v2d(&b, 8, 12, Floor), 11);

        assert_eq!(f.index_set_horizontal_v2d(&b, 11, 0, Floor), 0);
        assert_eq!(f.index_set_horizontal_v2d(&b, 11, 11, Floor), 11);
        assert_eq!(f.index_set_horizontal_v2d(&b, 11, 12, Floor), 11);

        assert_eq!(f.index_set_horizontal_v2d(&b, 12, 0, Floor), 12);
        assert_eq!(f.index_set_horizontal_v2d(&b, 12, 11, Floor), 23);
        assert_eq!(f.index_set_horizontal_v2d(&b, 12, 12, Floor), 23);

        assert_eq!(f.index_set_horizontal_v2d(&b, 17, 0, Floor), 12);
        assert_eq!(f.index_set_horizontal_v2d(&b, 17, 11, Floor), 23);
        assert_eq!(f.index_set_horizontal_v2d(&b, 17, 12, Floor), 23);

        assert_eq!(f.index_set_horizontal_v2d(&b, 23, 0, Floor), 12);
        assert_eq!(f.index_set_horizontal_v2d(&b, 23, 11, Floor), 23);
        assert_eq!(f.index_set_horizontal_v2d(&b, 23, 12, Floor), 23);
    }

    #[test]
    fn index_offset_vertical_v2d_1() {
        let b = Buffer::new_from_str("Hello there, stranger!\nHow are you doing this fine day?"); // 55 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(80);

        assert_eq!(f.index_offset_vertical_v2d(&b, 0, 0, (Floor, Floor)), 0);
        assert_eq!(f.index_offset_vertical_v2d(&b, 0, 1, (Floor, Floor)), 23);
        assert_eq!(f.index_offset_vertical_v2d(&b, 23, -1, (Floor, Floor)), 0);

        assert_eq!(f.index_offset_vertical_v2d(&b, 2, 0, (Floor, Floor)), 2);
        assert_eq!(f.index_offset_vertical_v2d(&b, 2, 1, (Floor, Floor)), 25);
        assert_eq!(f.index_offset_vertical_v2d(&b, 25, -1, (Floor, Floor)), 2);

        assert_eq!(f.index_offset_vertical_v2d(&b, 22, 0, (Floor, Floor)), 22);
        assert_eq!(f.index_offset_vertical_v2d(&b, 22, 1, (Floor, Floor)), 45);
        assert_eq!(f.index_offset_vertical_v2d(&b, 45, -1, (Floor, Floor)), 22);

        assert_eq!(f.index_offset_vertical_v2d(&b, 54, 0, (Floor, Floor)), 54);
        assert_eq!(f.index_offset_vertical_v2d(&b, 54, 1, (Floor, Floor)), 55);
        assert_eq!(f.index_offset_vertical_v2d(&b, 54, -1, (Floor, Floor)), 22);
    }

    #[test]
    fn index_offset_vertical_v2d_2() {
        let b = Buffer::new_from_str("Hello there, stranger! How are you doing this fine day?"); // 55 graphemes long

        let mut f = ConsoleLineFormatter::new(4);
        f.wrap_type = WrapType::CharWrap(0);
        f.maintain_indent = false;
        f.wrap_additional_indent = 0;
        f.set_wrap_width(12);

        assert_eq!(f.index_offset_vertical_v2d(&b, 0, 0, (Floor, Floor)), 0);
        assert_eq!(f.index_offset_vertical_v2d(&b, 0, 1, (Floor, Floor)), 12);
        assert_eq!(f.index_offset_vertical_v2d(&b, 0, 2, (Floor, Floor)), 24);

        assert_eq!(f.index_offset_vertical_v2d(&b, 0, 0, (Floor, Floor)), 0);
        assert_eq!(f.index_offset_vertical_v2d(&b, 12, -1, (Floor, Floor)), 0);
        assert_eq!(f.index_offset_vertical_v2d(&b, 24, -2, (Floor, Floor)), 0);

        assert_eq!(f.index_offset_vertical_v2d(&b, 4, 0, (Floor, Floor)), 4);
        assert_eq!(f.index_offset_vertical_v2d(&b, 4, 1, (Floor, Floor)), 16);
        assert_eq!(f.index_offset_vertical_v2d(&b, 4, 2, (Floor, Floor)), 28);

        assert_eq!(f.index_offset_vertical_v2d(&b, 4, 0, (Floor, Floor)), 4);
        assert_eq!(f.index_offset_vertical_v2d(&b, 16, -1, (Floor, Floor)), 4);
        assert_eq!(f.index_offset_vertical_v2d(&b, 28, -2, (Floor, Floor)), 4);

        assert_eq!(f.index_offset_vertical_v2d(&b, 11, 0, (Floor, Floor)), 11);
        assert_eq!(f.index_offset_vertical_v2d(&b, 11, 1, (Floor, Floor)), 23);
        assert_eq!(f.index_offset_vertical_v2d(&b, 11, 2, (Floor, Floor)), 35);

        assert_eq!(f.index_offset_vertical_v2d(&b, 11, 0, (Floor, Floor)), 11);
        assert_eq!(f.index_offset_vertical_v2d(&b, 23, -1, (Floor, Floor)), 11);
        assert_eq!(f.index_offset_vertical_v2d(&b, 35, -2, (Floor, Floor)), 11);
    }
}
