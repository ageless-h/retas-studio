use crate::Document;

#[derive(Debug, Clone)]
pub struct History {
    undo_stack: Vec<Document>,
    redo_stack: Vec<Document>,
    max_size: usize,
    is_recording: bool,
}

impl History {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
            is_recording: true,
        }
    }

    pub fn record(&mut self, state: &Document) {
        if !self.is_recording {
            return;
        }
        if self.undo_stack.len() >= self.max_size {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(state.clone());
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self, current: &Document) -> Option<Document> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current.clone());
        Some(prev)
    }

    pub fn redo(&mut self, current: &Document) -> Option<Document> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current.clone());
        Some(next)
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn disable_recording(&mut self) {
        self.is_recording = false;
    }

    pub fn enable_recording(&mut self) {
        self.is_recording = true;
    }
}
