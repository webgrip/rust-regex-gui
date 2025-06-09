#[derive(Default)]
pub struct Rule {
    pub from: String,
    pub to: String,
    pub match_count: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rule_is_empty() {
        let rule = Rule::default();
        assert!(rule.from.is_empty());
        assert!(rule.to.is_empty());
        assert!(rule.match_count.is_none());
    }
}
