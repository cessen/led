use buffer::line::{Line, LineGraphemeIter};
use std::cmp::min;

#[derive(Copy, PartialEq)]
pub enum RoundingBehavior {
    Round,
    Floor,
    Ceiling,
}


pub trait LineFormatter<'a> {
    type Iter: Iterator<Item=(&'a str, (usize, usize), (usize, usize))> + 'a;
    
    fn single_line_height(&self) -> usize;

    fn iter(&'a self, line: &'a Line) -> Self::Iter;
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
    type Item = (&'a str, (usize, usize), (usize, usize));
    
    fn next(&mut self) -> Option<(&'a str, (usize, usize), (usize, usize))> {
        if let Some(g) = self.grapheme_iter.next() {
            let pos = self.pos;
            self.pos = (pos.0, pos.1 + 1);
            return Some((g, pos, (1, self.f.tab_width as usize)));
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