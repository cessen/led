use buffer::line::{Line, LineGraphemeIter};
use std::cmp::{min, max};

#[derive(Copy, PartialEq)]
pub enum RoundingBehavior {
    Round,
    Floor,
    Ceiling,
}


pub trait LineFormatter<'a> {
    // The iterator yields the grapheme, the 2d position of the grapheme, and the grapheme's width
    type Iter: Iterator<Item=(&'a str, (usize, usize), usize)> + 'a;
    
    fn single_line_height(&self) -> usize;

    fn iter(&'a self, line: &'a Line) -> Self::Iter;
    
    
    /// Returns the 2d visual dimensions of the given line when formatted
    /// by the formatter.
    fn dimensions(&'a self, line: &'a Line) -> (usize, usize) {
        let mut dim: (usize, usize) = (0, 0);
        
        for (_, pos, width) in self.iter(line) {            
            dim = (max(dim.0, pos.0), max(dim.1, pos.1 + width));
        }
        
        dim.0 += self.single_line_height();
        
        return dim;
    }
    
    
    /// Converts a grapheme index within a line into a visual 2d position.
    fn index_to_v2d(&'a self, line: &'a Line, index: usize) -> (usize, usize) {
        let mut pos = (0, 0);
        let mut i = 0;
        let mut last_width = 0;
        
        for (_, _pos, width) in self.iter(line) {
            pos = _pos;
            last_width = width;
            i += 1;
            
            if i > index {
                return pos;
            }
        }
        
        return (pos.0, pos.1 + last_width);
    }
    
    
    /// Converts a visual 2d position into a grapheme index within a line.
    fn v2d_to_index(&'a self, line: &'a Line, v2d: (usize, usize), rounding: (RoundingBehavior, RoundingBehavior)) -> usize {
        // TODO: handle rounding modes
        let mut i = 0;
        
        for (_, pos, _) in self.iter(line) {
            if pos.0 > v2d.0 {
                break;
            }
            else if pos.0 == v2d.0 && pos.1 >= v2d.1 {
                break;
            }
            
            i += 1;
        }
        
        return i;
    }
}



//================================================================
// A simple implementation of LineFormatter, and LineFormatIter
// for testing purposes.
//================================================================

pub struct TestLineFormatIter<'a> {
    grapheme_iter: LineGraphemeIter<'a>,
    f: &'a TestLineFormatter,
    pos: (usize, usize),
}

impl<'a> Iterator for TestLineFormatIter<'a> {
    type Item = (&'a str, (usize, usize), usize);
    
    fn next(&mut self) -> Option<(&'a str, (usize, usize), usize)> {
        if let Some(g) = self.grapheme_iter.next() {
            let pos = self.pos;
            self.pos = (pos.0, pos.1 + 1);
            return Some((g, pos, 1));
        }
        else {
            return None;
        }
    }
}


pub struct TestLineFormatter {
    tab_width: u8
}

impl TestLineFormatter {
    pub fn new() -> TestLineFormatter {
        TestLineFormatter {
            tab_width: 4,
        }
    }
}

impl<'a> LineFormatter<'a> for TestLineFormatter {
    type Iter = TestLineFormatIter<'a>;
    
    fn single_line_height(&self) -> usize {
        1
    }
    
    fn iter(&'a self, line: &'a Line) -> TestLineFormatIter<'a> {
        TestLineFormatIter {
            grapheme_iter: line.grapheme_iter(),
            f: self,
            pos: (0, 0),
        }
    }
}




mod tests {
    use super::{LineFormatter, TestLineFormatter, TestLineFormatIter};
    use buffer::line::Line;
    
    #[test]
    fn simple_iterator() {
        let line = Line::new_from_str("Hello!");
        let mut f = TestLineFormatter::new();
        let mut iter = f.iter(&line);
        
        let (a,_,_) = iter.next().unwrap();
        assert_eq!(a, "H");
        
        let (a,_,_) = iter.next().unwrap();
        assert_eq!(a, "e");
        
        let (a,_,_) = iter.next().unwrap();
        assert_eq!(a, "l");
        
        let (a,_,_) = iter.next().unwrap();
        assert_eq!(a, "l");
        
        let (a,_,_) = iter.next().unwrap();
        assert_eq!(a, "o");
        
        let (a,_,_) = iter.next().unwrap();
        assert_eq!(a, "!");
        
        let a = iter.next();
        assert_eq!(a, None);
    }
}