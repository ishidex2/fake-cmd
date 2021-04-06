pub struct Screen {
    text: String,
    pub color: u8
}

impl Screen {
    pub fn new(color: u8) -> Self {
        Self {
            color,
            text: "".to_string()
        }
    }

    pub fn set_text(&mut self, string: String) {
        self.text = string.to_string()
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }
}
