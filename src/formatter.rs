#![allow(dead_code)]

use std::cmp::min;
use buffer::Buffer;

// Maximum graphemes in a line before a soft line break is forced.
// This is necessary to prevent pathological formatting cases which
// could slow down the editor arbitrarily for arbitrarily long
// lines.
pub const LINE_BLOCK_LENGTH: usize = 4096;


#[derive(Copy, PartialEq)]
pub enum RoundingBehavior {
    Round,
    Floor,
    Ceiling,
}


pub trait LineFormatter {
    fn single_line_height(&self) -> usize;
    
    /// Returns the 2d visual dimensions of the given text when formatted
    /// by the formatter.
    /// The text to be formatted is passed as a grapheme iterator.
    fn dimensions<'a, T>(&'a self, g_iter: T) -> (usize, usize)
    where T: Iterator<Item=&'a str>;
    
    
    /// Converts a grapheme index within a text into a visual 2d position.
    /// The text to be formatted is passed as a grapheme iterator.
    fn index_to_v2d<'a, T>(&'a self, g_iter: T, index: usize) -> (usize, usize)
    where T: Iterator<Item=&'a str>;
    
    
    /// Converts a visual 2d position into a grapheme index within a text.
    /// The text to be formatted is passed as a grapheme iterator.
    fn v2d_to_index<'a, T>(&'a self, g_iter: T, v2d: (usize, usize), rounding: (RoundingBehavior, RoundingBehavior)) -> usize
    where T: Iterator<Item=&'a str>;


    fn index_to_horizontal_v2d(&self, buf: &Buffer, index: usize) -> usize {
        let (line_i, col_i) = buf.index_to_line_col(index);
        let line = buf.get_line(line_i);
        
        // Find the right block in the line, and the index within that block
        let (line_block, col_i_adjusted) = block_index_and_offset(col_i);
        
        // Get an iter into the right block
        let g_iter = line.grapheme_iter_between_indices(line_block * LINE_BLOCK_LENGTH, (line_block+1) * LINE_BLOCK_LENGTH);
        return self.index_to_v2d(g_iter, col_i_adjusted).1;
    }
    
    
    /// Takes a grapheme index and a visual vertical offset, and returns the grapheme
    /// index after that visual offset is applied.
    fn index_offset_vertical_v2d(&self, buf: &Buffer, index: usize, offset: isize, rounding: (RoundingBehavior, RoundingBehavior)) -> usize {
        // TODO: handle rounding modes
        // TODO: do this with bidirectional line iterator
        
        // Get the line and block index of the given index
        let (mut line_i, mut col_i) = buf.index_to_line_col(index);
        
        // Find the right block in the line, and the index within that block
        let (line_block, col_i_adjusted) = block_index_and_offset(col_i);
        
        let (mut y, x) = self.index_to_v2d(buf.get_line(line_i).grapheme_iter_between_indices(line_block * LINE_BLOCK_LENGTH, (line_block+1) * LINE_BLOCK_LENGTH), col_i_adjusted);
        
        // First, find the right line while keeping track of the vertical offset
        let mut new_y = y as isize + offset;
        let mut line;
        let mut block_index: usize = line_block;
        loop {
            line = buf.get_line(line_i);
            let (h, _) = self.dimensions(line.grapheme_iter_between_indices(block_index * LINE_BLOCK_LENGTH, (block_index+1) * LINE_BLOCK_LENGTH));
            
            if new_y >= 0 && new_y < h as isize {
                y = new_y as usize;
                break;
            }
            else {
                if new_y > 0 {
                    let is_last_block = block_index >= last_block_index(line.grapheme_count());
                    
                    // Check for off-the-end
                    if is_last_block && (line_i + 1) >= buf.line_count() {
                        return buf.grapheme_count();
                    }
                    
                    if is_last_block { 
                        line_i += 1;
                        block_index = 0;
                    }
                    else {
                        block_index += 1;
                    }
                    new_y -= h as isize;
                }
                else if new_y < 0 {
                    // Check for off-the-end
                    if block_index == 0 && line_i == 0 {
                        return 0;
                    }
                    
                    if block_index == 0 {
                        line_i -= 1;
                        line = buf.get_line(line_i);
                        block_index = last_block_index(line.grapheme_count());
                    }
                    else {
                        block_index -= 1;
                    }
                    let (h, _) = self.dimensions(line.grapheme_iter_between_indices(block_index * LINE_BLOCK_LENGTH, (block_index+1) * LINE_BLOCK_LENGTH));
                    new_y += h as isize;
                }
                else {
                    unreachable!();
                }
            }
        }
        
        // Next, convert the resulting coordinates back into buffer-wide
        // coordinates.
        let block_slice = line.slice(block_index * LINE_BLOCK_LENGTH, (block_index+1) * LINE_BLOCK_LENGTH);
        let block_col_i = min(self.v2d_to_index(block_slice.grapheme_iter(), (y, x), rounding), LINE_BLOCK_LENGTH - 1);
        col_i = (block_index * LINE_BLOCK_LENGTH) + block_col_i;
        
        return buf.line_col_to_index((line_i, col_i));
    }
    
    
    /// Takes a grapheme index and a desired visual horizontal position, and
    /// returns a grapheme index on the same visual line as the given index,
    /// but offset to have the desired horizontal position.
    fn index_set_horizontal_v2d(&self, buf: &Buffer, index: usize, horizontal: usize, rounding: RoundingBehavior) -> usize {
        let (line_i, col_i) = buf.index_to_line_col(index);
        let line = buf.get_line(line_i);
        
        // Find the right block in the line, and the index within that block
        let (line_block, col_i_adjusted) = block_index_and_offset(col_i);
        let start_index = line_block * LINE_BLOCK_LENGTH;
        
        // Calculate the horizontal position
        let (v, _) = self.index_to_v2d(line.grapheme_iter_between_indices(start_index, start_index+LINE_BLOCK_LENGTH), col_i_adjusted);
        let block_col_i = self.v2d_to_index(line.grapheme_iter_between_indices(start_index, start_index+LINE_BLOCK_LENGTH), (v, horizontal), (RoundingBehavior::Floor, rounding));
        let mut new_col_i = start_index + min(block_col_i, LINE_BLOCK_LENGTH - 1);
        
        // Make sure we're not pushing the index off the end of the line
        if (line_i + 1) < buf.line_count()
        && new_col_i >= line.grapheme_count()
        && line.grapheme_count() > 0
        {
            new_col_i = line.grapheme_count() - 1;
        }
        
        return (index + new_col_i) - col_i;
    }
    
}

pub fn block_index_and_offset(index: usize) -> (usize, usize) {
    (index / LINE_BLOCK_LENGTH, index % LINE_BLOCK_LENGTH)
}

pub fn last_block_index(gc: usize) -> usize {
    let mut block_count = gc / LINE_BLOCK_LENGTH;
    if (gc % LINE_BLOCK_LENGTH) > 0 {
        block_count += 1;
    }
    
    if block_count > 0 {
        return block_count - 1;
    }
    else {
        return 0;
    }
}
