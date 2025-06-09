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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestLogger {
        messages: Arc<Mutex<Vec<String>>>,
    }

    impl Logger for TestLogger {
        fn log(&self, message: &str) {
            self.messages.lock().unwrap().push(message.to_string());
        }
    }

    #[test]
    fn execute_logs_each_mapping() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let logger = Arc::new(TestLogger {
            messages: Arc::clone(&messages),
        });
        let renamer = Renamer::new(logger);

        let rules = vec![
            Rule {
                from: "src".into(),
                to: "dst".into(),
            },
            Rule {
                from: "foo".into(),
                to: "bar".into(),
            },
        ];

        renamer.execute(&rules);

        let collected = messages.lock().unwrap().clone();
        assert_eq!(
            collected,
            vec![
                "Mapping 'src' -> 'dst'".to_string(),
                "Mapping 'foo' -> 'bar'".to_string(),
            ]
        );
    }
}
