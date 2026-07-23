use image::DynamicImage;

#[derive(Debug, Clone, PartialEq)]
pub enum LayerType {
    SmartCrop(String),
    Enhance(f32),
    TuneImage { warmth: f32, vignette: f32, structure: f32 },
    HDRScape,
    GlamourGlow,
    HazeRemoval,
    Grayscale,
    Sepia,
    Invert,
    Contrast(f32),
    Watermark(String),
    Frames(u32),
}

#[derive(Debug, Clone)]
pub struct EditLayer {
    pub id: usize,
    pub name: String,
    pub enabled: bool,
    pub layer_type: LayerType,
}

pub struct EditSession {
    pub initial_image: DynamicImage,
    pub current_image: DynamicImage,
    pub undo_stack: Vec<DynamicImage>,
    pub redo_stack: Vec<DynamicImage>,
    pub history_log: Vec<String>,
    pub layers: Vec<EditLayer>,
    next_layer_id: usize,
}

impl EditSession {
    pub fn new(initial_image: DynamicImage) -> Self {
        Self {
            initial_image: initial_image.clone(),
            current_image: initial_image,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            history_log: vec!["Initial Base Image".to_string()],
            layers: Vec::new(),
            next_layer_id: 1,
        }
    }

    pub fn apply_action(&mut self, new_image: DynamicImage, description: String) {
        self.undo_stack.push(self.current_image.clone());
        self.redo_stack.clear();
        self.current_image = new_image;
        self.history_log.push(description);
    }

    pub fn add_layer(&mut self, name: String, layer_type: LayerType) {
        let layer = EditLayer {
            id: self.next_layer_id,
            name,
            enabled: true,
            layer_type,
        };
        self.next_layer_id += 1;
        self.layers.push(layer);
    }

    pub fn remove_layer_at(&mut self, index: usize) -> bool {
        if index < self.layers.len() {
            self.layers.remove(index);
            true
        } else {
            false
        }
    }

    pub fn toggle_layer_at(&mut self, index: usize) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.enabled = !layer.enabled;
        }
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
