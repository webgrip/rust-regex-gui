use regex::Regex;

#[derive(Debug, Clone)]
pub struct Rule {
    pattern: Regex,
    to: String,
}

impl Rule {
    pub fn new(pattern: &str, to: impl Into<String>) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
            to: to.into(),
        })
    }

    pub fn apply(&self, input: &str) -> Option<String> {
        if self.pattern.is_match(input) {
            Some(self.to.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_matches_input() {
        let rule = Rule::new("^file.*\\.txt$", "dest").unwrap();
        assert_eq!(rule.apply("file1.txt"), Some("dest".into()));
    }

    #[test]
    fn rule_does_not_match() {
        let rule = Rule::new("^file.*\\.txt$", "dest").unwrap();
        assert_eq!(rule.apply("other.pdf"), None);
    }
}
