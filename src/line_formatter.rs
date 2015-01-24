use buffer::line::{Line, LineGraphemeIter};
use string_utils::{is_line_ending};

pub enum RoundingBehavior {
    Round,
    Floor,
    Ceiling,
}

pub trait LineFormatter {
    fn dimensions(&self, line: &Line) -> (usize, usize);
    
    fn index_to_v2d(&self, line: &Line, index: usize) -> (usize, usize);
    
    fn v2d_to_index(&self, line: &Line, v2d: (usize, usize), rounding: (RoundingBehavior, RoundingBehavior)) -> usize;
}





//============================================================
// An implementation of the LineFormatter stuff for consoles

pub struct ConsoleLineFormatterVisIter<'a> {
    grapheme_iter: LineGraphemeIter<'a>,
    f: &'a ConsoleLineFormatter,
    pos: (usize, usize),
}



impl<'a> Iterator for ConsoleLineFormatterVisIter<'a> {
    type Item = (&'a str, usize, usize);

    fn next(&mut self) -> Option<(&'a str, usize, usize)> {
        if let Some(g) = self.grapheme_iter.next() {
            let pos = self.pos;
            let width = grapheme_vis_width_at_vis_pos(g, self.pos.1, self.f.tab_width as usize);
            self.pos = (self.pos.0, self.pos.1 + width);
            return Some((g, pos.0, pos.1));
        }
        else {
            return None;
        }
    }
}


pub struct ConsoleLineFormatter {
    pub tab_width: u8,
}


impl ConsoleLineFormatter {
    pub fn new(tab_width: u8) -> ConsoleLineFormatter {
        ConsoleLineFormatter {
            tab_width: tab_width,
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
    fn dimensions(&self, line: &Line) -> (usize, usize) {
        return (1, self.vis_width(line));
    }
    
    
    fn index_to_v2d(&self, line: &Line, index: usize) -> (usize, usize) {
        let mut pos = 0;
        let mut iter = line.grapheme_iter();
        
        for _ in range(0, index) {
            if let Some(g) = iter.next() {
                let w = grapheme_vis_width_at_vis_pos(g, pos, self.tab_width as usize);
                pos += w;
            }
            else {
                panic!("ConsoleLineFormatter::index_to_v2d(): index past end of line.");
            }
        }
        
        return (0, pos);
    }
    
    
    fn v2d_to_index(&self, line: &Line, v2d: (usize, usize), rounding: (RoundingBehavior, RoundingBehavior)) -> usize {
        let mut pos = 0;
        let mut i = 0;
        let mut iter = line.grapheme_iter();
        
        while pos < v2d.1 {
            if let Some(g) = iter.next() {
                let w = grapheme_vis_width_at_vis_pos(g, pos, self.tab_width as usize);
                if (w + pos) > v2d.1 {
                    let d1 = v2d.1 - pos;
                    let d2 = (pos + w) - v2d.1;
                    if d2 < d1 {
                        i += 1;
                    }
                    break;
                }
                else {
                    pos += w;
                    i += 1;
                }
            }
            else {
                break;
            }
        }
        
        return i;
    }
}



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