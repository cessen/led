// use std::{borrow::Cow, cmp::max};

// use crate::{
//     formatter::{LineFormatter, RoundingBehavior},
//     string_utils::{is_line_ending, is_whitespace},
//     utils::{grapheme_width, RopeGraphemes},
// };

// #[cfg(test)]
// mod tests {
//     #![allow(unused_imports)]
//     use ropey::Rope;

//     use crate::buffer::Buffer;
//     use crate::formatter::LineFormatter;
//     use crate::formatter::RoundingBehavior::{Ceiling, Floor, Round};
//     use crate::utils::RopeGraphemes;

//     use super::*;

//     #[test]
//     fn dimensions_1() {
//         let text = Rope::from_str("Hello there, stranger!"); // 22 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 80;
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(f.dimensions(RopeGraphemes::new(&text.slice(..))), (1, 22));
//     }

//     #[test]
//     fn dimensions_3() {
//         let text = Rope::from_str("Hello there, stranger!  How are you doing this fine day?"); // 56 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 12;
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(f.dimensions(RopeGraphemes::new(&text.slice(..))), (6, 12));
//     }

//     #[test]
//     fn dimensions_4() {
//         // 55 graphemes long
//         let text = Rope::from_str(
//             "税マイミ文末\
//              レ日題イぽじ\
//              や男目統ス公\
//              身みトしつ結\
//              煮ヱマレ断西\
//              ロ領視りいぽ\
//              凱字テ式重反\
//              てす献罪がご\
//              く官俵呉嫁ー\
//              。",
//         );

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 12;
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(f.dimensions(RopeGraphemes::new(&text.slice(..))), (10, 12));
//     }

//     #[test]
//     fn dimensions_5() {
//         // 55 graphemes long
//         let text = Rope::from_str(
//             "税マイミ文末\
//              レ日題イぽじ\
//              や男目統ス公\
//              身みトしつ結\
//              煮ヱマレ断西\
//              ロ領視りいぽ\
//              凱字テ式重反\
//              てす献罪がご\
//              く官俵呉嫁ー\
//              。",
//         );

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 12;
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(f.dimensions(RopeGraphemes::new(&text.slice(..))), (10, 12));
//     }

//     #[test]
//     fn index_to_v2d_1() {
//         let text = Rope::from_str("Hello there, stranger!"); // 22 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 80;
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 0),
//             (0, 0)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 5),
//             (0, 5)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 22),
//             (0, 22)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 23),
//             (0, 22)
//         );
//     }

//     #[test]
//     fn index_to_v2d_2() {
//         let text = Rope::from_str("Hello there, stranger!  How are you doing this fine day?"); // 56 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 12; // Was char wrap.
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 0),
//             (0, 0)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 5),
//             (0, 5)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 11),
//             (0, 11)
//         );

//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 12),
//             (1, 0)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 15),
//             (1, 3)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 23),
//             (1, 11)
//         );

//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 24),
//             (2, 0)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 28),
//             (2, 4)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 35),
//             (2, 11)
//         );

//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 36),
//             (3, 0)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 43),
//             (3, 7)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 47),
//             (3, 11)
//         );

//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 48),
//             (4, 0)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 50),
//             (4, 2)
//         );
//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 56),
//             (4, 8)
//         );

//         assert_eq!(
//             f.index_to_v2d(RopeGraphemes::new(&text.slice(..)), 57),
//             (4, 8)
//         );
//     }

//     #[test]
//     fn v2d_to_index_1() {
//         let text = Rope::from_str("Hello there, stranger!"); // 22 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 80;
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 0), (Floor, Floor)),
//             0
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 5), (Floor, Floor)),
//             5
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 22), (Floor, Floor)),
//             22
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 23), (Floor, Floor)),
//             22
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (1, 0), (Floor, Floor)),
//             22
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (1, 1), (Floor, Floor)),
//             22
//         );
//     }

//     #[test]
//     fn v2d_to_index_2() {
//         let text = Rope::from_str("Hello there, stranger!  How are you doing this fine day?"); // 56 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 12; // Was char wrap.
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 0), (Floor, Floor)),
//             0
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 11), (Floor, Floor)),
//             11
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (0, 12), (Floor, Floor)),
//             11
//         );

//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (1, 0), (Floor, Floor)),
//             12
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (1, 11), (Floor, Floor)),
//             23
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (1, 12), (Floor, Floor)),
//             23
//         );

//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (2, 0), (Floor, Floor)),
//             24
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (2, 11), (Floor, Floor)),
//             35
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (2, 12), (Floor, Floor)),
//             35
//         );

//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (3, 0), (Floor, Floor)),
//             36
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (3, 11), (Floor, Floor)),
//             47
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (3, 12), (Floor, Floor)),
//             47
//         );

//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (4, 0), (Floor, Floor)),
//             48
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (4, 7), (Floor, Floor)),
//             55
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (4, 8), (Floor, Floor)),
//             56
//         );
//         assert_eq!(
//             f.v2d_to_index(RopeGraphemes::new(&text.slice(..)), (4, 9), (Floor, Floor)),
//             56
//         );
//     }

//     #[test]
//     fn index_to_horizontal_v2d_1() {
//         let b = Buffer::new_from_str("Hello there, stranger!\nHow are you doing this fine day?"); // 55 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 80;
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(f.index_to_horizontal_v2d(&b, 0), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 5), 5);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 26), 3);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 55), 32);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 56), 32);
//     }

//     #[test]
//     fn index_to_horizontal_v2d_2() {
//         let b = Buffer::new_from_str("Hello there, stranger!\nHow are you doing this fine day?"); // 55 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 12; // Was char wrap.
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(f.index_to_horizontal_v2d(&b, 0), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 11), 11);

//         assert_eq!(f.index_to_horizontal_v2d(&b, 12), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 22), 10);

//         assert_eq!(f.index_to_horizontal_v2d(&b, 23), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 34), 11);

//         assert_eq!(f.index_to_horizontal_v2d(&b, 35), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 46), 11);

//         assert_eq!(f.index_to_horizontal_v2d(&b, 47), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 55), 8);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 56), 8);
//     }

//     #[test]
//     fn index_to_horizontal_v2d_3() {
//         let b = Buffer::new_from_str("Hello there, stranger!\nHow are you doing this fine day?"); // 55 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 12;
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(f.index_to_horizontal_v2d(&b, 0), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 5), 5);

//         assert_eq!(f.index_to_horizontal_v2d(&b, 6), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 12), 6);

//         assert_eq!(f.index_to_horizontal_v2d(&b, 13), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 22), 9);

//         assert_eq!(f.index_to_horizontal_v2d(&b, 23), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 34), 11);

//         assert_eq!(f.index_to_horizontal_v2d(&b, 35), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 45), 10);

//         assert_eq!(f.index_to_horizontal_v2d(&b, 46), 0);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 55), 9);
//         assert_eq!(f.index_to_horizontal_v2d(&b, 56), 9);
//     }

//     #[test]
//     fn index_set_horizontal_v2d_1() {
//         let b = Buffer::new_from_str("Hello there, stranger!\nHow are you doing this fine day?"); // 55 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 80;
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(f.index_set_horizontal_v2d(&b, 0, 0, Floor), 0);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 0, 22, Floor), 22);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 0, 23, Floor), 22);

//         assert_eq!(f.index_set_horizontal_v2d(&b, 8, 0, Floor), 0);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 8, 22, Floor), 22);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 8, 23, Floor), 22);

//         assert_eq!(f.index_set_horizontal_v2d(&b, 22, 0, Floor), 0);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 22, 22, Floor), 22);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 22, 23, Floor), 22);

//         assert_eq!(f.index_set_horizontal_v2d(&b, 23, 0, Floor), 23);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 23, 32, Floor), 55);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 23, 33, Floor), 55);

//         assert_eq!(f.index_set_horizontal_v2d(&b, 28, 0, Floor), 23);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 28, 32, Floor), 55);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 28, 33, Floor), 55);

//         assert_eq!(f.index_set_horizontal_v2d(&b, 55, 0, Floor), 23);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 55, 32, Floor), 55);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 55, 33, Floor), 55);
//     }

//     #[test]
//     fn index_set_horizontal_v2d_2() {
//         let b = Buffer::new_from_str("Hello there, stranger! How are you doing this fine day?"); // 55 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 12; // Was char wrap.
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(f.index_set_horizontal_v2d(&b, 0, 0, Floor), 0);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 0, 11, Floor), 11);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 0, 12, Floor), 11);

//         assert_eq!(f.index_set_horizontal_v2d(&b, 8, 0, Floor), 0);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 8, 11, Floor), 11);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 8, 12, Floor), 11);

//         assert_eq!(f.index_set_horizontal_v2d(&b, 11, 0, Floor), 0);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 11, 11, Floor), 11);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 11, 12, Floor), 11);

//         assert_eq!(f.index_set_horizontal_v2d(&b, 12, 0, Floor), 12);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 12, 11, Floor), 23);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 12, 12, Floor), 23);

//         assert_eq!(f.index_set_horizontal_v2d(&b, 17, 0, Floor), 12);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 17, 11, Floor), 23);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 17, 12, Floor), 23);

//         assert_eq!(f.index_set_horizontal_v2d(&b, 23, 0, Floor), 12);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 23, 11, Floor), 23);
//         assert_eq!(f.index_set_horizontal_v2d(&b, 23, 12, Floor), 23);
//     }

//     #[test]
//     fn index_offset_vertical_v2d_1() {
//         let b = Buffer::new_from_str("Hello there, stranger!\nHow are you doing this fine day?"); // 55 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 80; // Was char wrap.
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(f.index_offset_vertical_v2d(&b, 0, 0, (Floor, Floor)), 0);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 0, 1, (Floor, Floor)), 23);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 23, -1, (Floor, Floor)), 0);

//         assert_eq!(f.index_offset_vertical_v2d(&b, 2, 0, (Floor, Floor)), 2);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 2, 1, (Floor, Floor)), 25);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 25, -1, (Floor, Floor)), 2);

//         assert_eq!(f.index_offset_vertical_v2d(&b, 22, 0, (Floor, Floor)), 22);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 22, 1, (Floor, Floor)), 45);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 45, -1, (Floor, Floor)), 22);

//         assert_eq!(f.index_offset_vertical_v2d(&b, 54, 0, (Floor, Floor)), 54);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 54, 1, (Floor, Floor)), 55);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 54, -1, (Floor, Floor)), 22);
//     }

//     #[test]
//     fn index_offset_vertical_v2d_2() {
//         let b = Buffer::new_from_str("Hello there, stranger! How are you doing this fine day?"); // 55 graphemes long

//         let mut f = ConsoleLineFormatter::new(4);
//         f.wrap_width = 12;
//         f.maintain_indent = false;
//         f.wrap_additional_indent = 0;

//         assert_eq!(f.index_offset_vertical_v2d(&b, 0, 0, (Floor, Floor)), 0);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 0, 1, (Floor, Floor)), 12);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 0, 2, (Floor, Floor)), 24);

//         assert_eq!(f.index_offset_vertical_v2d(&b, 0, 0, (Floor, Floor)), 0);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 12, -1, (Floor, Floor)), 0);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 24, -2, (Floor, Floor)), 0);

//         assert_eq!(f.index_offset_vertical_v2d(&b, 4, 0, (Floor, Floor)), 4);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 4, 1, (Floor, Floor)), 16);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 4, 2, (Floor, Floor)), 28);

//         assert_eq!(f.index_offset_vertical_v2d(&b, 4, 0, (Floor, Floor)), 4);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 16, -1, (Floor, Floor)), 4);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 28, -2, (Floor, Floor)), 4);

//         assert_eq!(f.index_offset_vertical_v2d(&b, 11, 0, (Floor, Floor)), 11);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 11, 1, (Floor, Floor)), 23);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 11, 2, (Floor, Floor)), 35);

//         assert_eq!(f.index_offset_vertical_v2d(&b, 11, 0, (Floor, Floor)), 11);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 23, -1, (Floor, Floor)), 11);
//         assert_eq!(f.index_offset_vertical_v2d(&b, 35, -2, (Floor, Floor)), 11);
//     }
// }
