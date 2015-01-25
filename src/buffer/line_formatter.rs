use buffer::line::Line;
use std::cmp::min;

#[derive(Copy, PartialEq)]
pub enum RoundingBehavior {
    Round,
    Floor,
    Ceiling,
}


pub trait LineFormatter {
    fn single_line_height(&self) -> usize;

    fn dimensions(&self, line: &Line) -> (usize, usize);
    
    fn index_to_v2d(&self, line: &Line, index: usize) -> (usize, usize);
    
    fn v2d_to_index(&self, line: &Line, v2d: (usize, usize), rounding: (RoundingBehavior, RoundingBehavior)) -> usize;
}



//================================================================
// A simple implementation of LineFormatter, for testing purposes
//================================================================

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

impl LineFormatter for TestLineFormatter {
    fn single_line_height(&self) -> usize {
        1
    }

    fn dimensions(&self, line: &Line) -> (usize, usize) {
        (1, line.grapheme_count())
    }
    
    fn index_to_v2d(&self, line: &Line, index: usize) -> (usize, usize) {
        (0, min(line.grapheme_count(), index))
    }
    
    fn v2d_to_index(&self, line: &Line, v2d: (usize, usize), rounding: (RoundingBehavior, RoundingBehavior)) -> usize {
        if v2d.0 > 0 {
            line.grapheme_count()
        }
        else {
            min(line.grapheme_count(), v2d.1)
        }
    }
}