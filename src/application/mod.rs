use std::sync::Arc;

use crate::domain::Rule;
use crate::telemetry::Logger;

pub struct Renamer {
    logger: Arc<dyn Logger>,
}

impl Renamer {
    pub fn new(logger: Arc<dyn Logger>) -> Self {
        Self { logger }
    }

    pub fn execute(&self, rules: &[Rule]) {
        for rule in rules {
            self.logger
                .log(&format!("Mapping '{}' -> '{}'", rule.from, rule.to));
        }
    }
}
