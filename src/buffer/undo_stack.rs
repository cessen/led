use std::collections::DList;


/// A text editing operation
#[derive(Clone)]
pub enum Operation {
    InsertText(String, usize),
    RemoveTextBefore(String, usize),
    RemoveTextAfter(String, usize),
    MoveText(usize, usize, usize),
    CompositeOp(Vec<Operation>),
}


/// An undo/redo stack of text editing operations
pub struct UndoStack {
    stack_a: DList<Operation>,
    stack_b: DList<Operation>,
}

impl UndoStack {
    pub fn new() -> UndoStack {
        UndoStack {
            stack_a: DList::new(),
            stack_b: DList::new(),
        }
    }
    
    
    pub fn push(&mut self, op: Operation) {
        self.stack_a.push_back(op);
        self.stack_b.clear();
    }
    
    
    pub fn prev(&mut self) -> Option<Operation> {
        if let Some(op) = self.stack_a.pop_back() {
            self.stack_b.push_back(op.clone());
            return Some(op);
        }
        else {
            return None;
        }
    }
    
    
    pub fn next(&mut self) -> Option<Operation> {
        if let Some(op) = self.stack_b.pop_back() {
            self.stack_a.push_back(op.clone());
            return Some(op);
        }
        else {
            return None;
        }
    }
}