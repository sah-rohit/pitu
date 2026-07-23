use image::DynamicImage;

pub struct EditSession {
    pub current_image: DynamicImage,
    pub undo_stack: Vec<DynamicImage>,
    pub redo_stack: Vec<DynamicImage>,
    pub history_log: Vec<String>,
}

impl EditSession {
    pub fn new(initial_image: DynamicImage) -> Self {
        Self {
            current_image: initial_image,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            history_log: vec!["Initial Image Loaded".to_string()],
        }
    }

    pub fn apply_action(&mut self, new_image: DynamicImage, description: String) {
        self.undo_stack.push(self.current_image.clone());
        self.redo_stack.clear();
        self.current_image = new_image;
        self.history_log.push(description);
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.current_image.clone());
            self.current_image = prev;
            if self.history_log.len() > 1 {
                self.history_log.pop();
            }
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.current_image.clone());
            self.current_image = next;
            self.history_log.push("Redo Operation".to_string());
            true
        } else {
            false
        }
    }
}
