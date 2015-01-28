use std::cmp::max;

use string_utils::{is_line_ending};
use buffer::line::{Line, LineGraphemeIter};
use buffer::line_formatter::{LineFormatter, RoundingBehavior};

//===================================================================
// LineFormatter implementation for terminals/consoles.
//===================================================================

pub struct ConsoleLineFormatter {
    pub tab_width: u8,
    pub wrap_width: usize,
}


impl ConsoleLineFormatter {
    pub fn new(tab_width: u8) -> ConsoleLineFormatter {
        ConsoleLineFormatter {
            tab_width: tab_width,
            wrap_width: 40,
        }
    }


    /// Returns the visual cell width of a line
    pub fn vis_width(&self, line: &Line) -> usize {
        let mut width = 0;
        
        for g in line.grapheme_iter() {
            let w = grapheme_vis_width_at_vis_pos(g, width, self.tab_width as usize);
            width += w;
        }
        
        return width;
    }


    pub fn vis_grapheme_iter<'b>(&'b self, line: &'b Line) -> ConsoleLineFormatterVisIter<'b> {
        ConsoleLineFormatterVisIter {
            grapheme_iter: line.grapheme_iter(),
            f: self,
            pos: (0, 0),
        }
    }
}


impl<'a> LineFormatter for ConsoleLineFormatter {
    fn single_line_height(&self) -> usize {
        return 1;
    }

    fn dimensions(&self, line: &Line) -> (usize, usize) {
        let mut dim: (usize, usize) = (0, 0);
        
        for (_, pos, width) in self.vis_grapheme_iter(line) {            
            dim = (max(dim.0, pos.0), max(dim.1, pos.1 + width));
        }
        
        dim.0 += 1;
        
        return dim;
    }
    
    
    fn index_to_v2d(&self, line: &Line, index: usize) -> (usize, usize) {
        let mut pos = (0, 0);
        let mut i = 0;
        
        for (_, _pos, _) in self.vis_grapheme_iter(line) {
            pos = _pos;
            i += 1;
            
            if i > index {
                break;
            }
        }
        
        return pos;
    }
    
    
    fn v2d_to_index(&self, line: &Line, v2d: (usize, usize), rounding: (RoundingBehavior, RoundingBehavior)) -> usize {
        // TODO: handle rounding modes
        let mut i = 0;
        
        for (_, pos, _) in self.vis_grapheme_iter(line) {
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


//===================================================================
// An iterator that iterates over the graphemes in a line in a
// manner consistent with the ConsoleFormatter.
//===================================================================
pub struct ConsoleLineFormatterVisIter<'a> {
    grapheme_iter: LineGraphemeIter<'a>,
    f: &'a ConsoleLineFormatter,
    pos: (usize, usize),
}



impl<'a> Iterator for ConsoleLineFormatterVisIter<'a> {
    type Item = (&'a str, (usize, usize), usize);

    fn next(&mut self) -> Option<(&'a str, (usize, usize), usize)> {
        if let Some(g) = self.grapheme_iter.next() {            
            let width = grapheme_vis_width_at_vis_pos(g, self.pos.1, self.f.tab_width as usize);
            
            if (self.pos.1 + width) > self.f.wrap_width {
                let pos = (self.pos.0 + 1, 0);
                self.pos = (self.pos.0 + 1, width);
                return Some((g, pos, width));
            }
            else {
                let pos = self.pos;
                self.pos = (self.pos.0, self.pos.1 + width);
                return Some((g, pos, width));
            }
        }
        else {
            return None;
        }
    }
}



//===================================================================
// Helper functions
//===================================================================

/// Returns the visual width of a grapheme given a starting
/// position on a line.
fn grapheme_vis_width_at_vis_pos(g: &str, pos: usize, tab_width: usize) -> usize {
    match g {
        "\t" => {
            let ending_pos = ((pos / tab_width) + 1) * tab_width;
            return ending_pos - pos;
        },
        
        _ => {
            if is_line_ending(g) {
                return 1;
            }
            else {
                return g.width(true);
            }
        }
    }
}
