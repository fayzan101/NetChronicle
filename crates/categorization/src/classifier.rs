use netchronicle_common::ActivityCategory;

use crate::RuleStore;

pub struct Categorizer {
    rules: RuleStore,
}

impl Categorizer {
    pub fn new(rules: RuleStore) -> Self {
        Self { rules }
    }

    pub fn classify_domain(&self, domain: &str) -> ActivityCategory {
        for rule in self.rules.rules() {
            if domain.contains(&rule.pattern) {
                return rule.category;
            }
        }
        ActivityCategory::Unknown
    }
}
