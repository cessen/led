#![allow(dead_code)]

use buffer::line::Line;
use buffer::Buffer;

#[derive(Copy, PartialEq)]
pub enum RoundingBehavior {
    Round,
    Floor,
    Ceiling,
}


pub trait LineFormatter {
    fn single_line_height(&self) -> usize;
    
    /// Returns the 2d visual dimensions of the given line when formatted
    /// by the formatter.
    fn dimensions(&self, line: &Line) -> (usize, usize);
    
    
    /// Converts a grapheme index within a line into a visual 2d position.
    fn index_to_v2d(&self, line: &Line, index: usize) -> (usize, usize);
    
    
    /// Converts a visual 2d position into a grapheme index within a line.
    fn v2d_to_index(&self, line: &Line, v2d: (usize, usize), rounding: (RoundingBehavior, RoundingBehavior)) -> usize;


    fn index_to_horizontal_v2d(&self, buf: &Buffer, index: usize) -> usize {
        let (line_i, col_i) = buf.index_to_line_col(index);
        let line = buf.get_line(line_i);
        return self.index_to_v2d(line, col_i).1;
    }
    
    
    /// Takes a grapheme index and a visual vertical offset, and returns the grapheme
    /// index after that visual offset is applied.
    fn index_offset_vertical_v2d(&self, buf: &Buffer, index: usize, offset: isize, rounding: (RoundingBehavior, RoundingBehavior)) -> usize {
        // TODO: handle rounding modes
        // TODO: do this with bidirectional line iterator
        let (mut line_i, mut col_i) = buf.index_to_line_col(index);
        let (mut y, x) = self.index_to_v2d(buf.get_line(line_i), col_i);
        let mut new_y = y as isize + offset;
        
        // First, find the right line while keeping track of the vertical offset
        let mut line;
        loop {
            line = buf.get_line(line_i);
            let (h, _) = self.dimensions(line);
            
            if new_y >= 0 && new_y < h as isize {
                y = new_y as usize;
                break;
            }
            else {
                if new_y > 0 {
                    // Check for off-the-end
                    if (line_i + 1) >= buf.line_count() {
                        return buf.grapheme_count();
                    }
                    
                    line_i += 1;
                    new_y -= h as isize;
                }
                else if new_y < 0 {
                    // Check for off-the-end
                    if line_i == 0 {
                        return 0;
                    }
                    
                    line_i -= 1;
                    line = buf.get_line(line_i);
                    let (h, _) = self.dimensions(line);
                    new_y += h as isize;
                }
                else {
                    unreachable!();
                }
            }
        }
        
        // Next, convert the resulting coordinates back into buffer-wide
        // coordinates.
        col_i = self.v2d_to_index(line, (y, x), rounding);
        
        return buf.line_col_to_index((line_i, col_i));
    }
    
    
    fn index_set_horizontal_v2d(&self, buf: &Buffer, index: usize, horizontal: usize, rounding: RoundingBehavior) -> usize {
        let (line_i, col_i) = buf.index_to_line_col(index);
        let line = buf.get_line(line_i);
        
        let (v, _) = self.index_to_v2d(line, col_i);
        let mut new_col_i = self.v2d_to_index(line, (v, horizontal), (RoundingBehavior::Floor, rounding));
        if new_col_i >= line.grapheme_count() && line.grapheme_count() > 0 {
            new_col_i = line.grapheme_count() - 1;
        }
        
        return (index + new_col_i) - col_i;
    }
    
}




//====================================================================
// UNIT TESTS
//====================================================================

//#[cfg(test)]
//mod tests {
//    #![allow(unused_imports)]
//    use buffer::line::{Line, LineGraphemeIter};
//    use super::LineFormatter;
//    
//    pub struct TestLineFormatIter<'a> {
//        grapheme_iter: LineGraphemeIter<'a>,
//        f: &'a TestLineFormatter,
//        pos: (usize, usize),
//    }
//    
//    impl<'a> Iterator for TestLineFormatIter<'a> {
//        type Item = (&'a str, (usize, usize), usize);
//        
//        fn next(&mut self) -> Option<(&'a str, (usize, usize), usize)> {
//            if let Some(g) = self.grapheme_iter.next() {
//                let pos = self.pos;
//                self.pos = (pos.0, pos.1 + 1);
//                return Some((g, pos, 1));
//            }
//            else {
//                return None;
//            }
//        }
//    }
//    
//    pub struct TestLineFormatter {
//        tab_width: u8
//    }
//    
//    impl TestLineFormatter {
//        pub fn new() -> TestLineFormatter {
//            TestLineFormatter {
//                tab_width: 4,
//            }
//        }
//    }
//    
//    impl<'a> LineFormatter<'a, TestLineFormatIter<'a>> for TestLineFormatter {
//        fn single_line_height(&self) -> usize {
//            1
//        }
//        
//        fn iter(&'a self, line: &'a Line) -> TestLineFormatIter<'a> {
//            TestLineFormatIter {
//                grapheme_iter: line.grapheme_iter(),
//                f: self,
//                pos: (0, 0),
//            }
//        }
//    }
//    
//    
//    #[test]
//    fn simple_iterator() {
//        let line = Line::new_from_str("Hello!");
//        let mut f = TestLineFormatter::new();
//        let mut iter = f.iter(&line);
//        
//        let (a,_,_) = iter.next().unwrap();
//        assert_eq!(a, "H");
//        
//        let (a,_,_) = iter.next().unwrap();
//        assert_eq!(a, "e");
//        
//        let (a,_,_) = iter.next().unwrap();
//        assert_eq!(a, "l");
//        
//        let (a,_,_) = iter.next().unwrap();
//        assert_eq!(a, "l");
//        
//        let (a,_,_) = iter.next().unwrap();
//        assert_eq!(a, "o");
//        
//        let (a,_,_) = iter.next().unwrap();
//        assert_eq!(a, "!");
//        
//        let a = iter.next();
//        assert_eq!(a, None);
//    }
//}//