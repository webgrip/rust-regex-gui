pub struct Rule {
    pub from: String,
    pub to: String,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            from: String::new(),
            to: String::new(),
        }
    }
}
