#[derive(Debug, Clone)]
pub struct History {
    edits: Vec<Edit>,
    position: usize, // Where we are in the history.
}

impl History {
    pub fn new() -> History {
        History {
            edits: Vec::new(),
            position: 0,
        }
    }

    pub fn push_edit(&mut self, edit: Edit) {
        self.edits.truncate(self.position);
        self.edits.push(edit);
    }

    pub fn undo(&mut self) -> Option<&Edit> {
        if self.position > 0 {
            self.position -= 1;
            Some(&self.edits[self.position])
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&Edit> {
        if self.position < self.edits.len() {
            let edit = &self.edits[self.position];
            self.position += 1;
            Some(edit)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Edit {
    pub char_idx: usize,
    pub from: String,
    pub to: String,
}
