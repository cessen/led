use std::cmp::max;

use string_utils::{is_line_ending};
use buffer::line::{Line, LineGraphemeIter};
use formatter::LineFormatter;

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
}


impl<'a> LineFormatter<'a> for ConsoleLineFormatter {
    type Iter = ConsoleLineFormatterVisIter<'a>;

    fn single_line_height(&self) -> usize {
        return 1;
    }
    
    fn iter(&'a self, line: &'a Line) -> ConsoleLineFormatterVisIter<'a> {
        ConsoleLineFormatterVisIter {
            grapheme_iter: line.grapheme_iter(),
            f: self,
            pos: (0, 0),
        }
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
                let pos = (self.pos.0 + self.f.single_line_height(), 0);
                self.pos = (self.pos.0 + self.f.single_line_height(), width);
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
