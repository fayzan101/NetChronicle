use netchronicle_common::ActivityCategory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRule {
    pub pattern: String,
    pub pattern_type: RulePatternType,
    pub category: ActivityCategory,
    pub priority: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RulePatternType {
    Domain,
    UrlPrefix,
    AppName,
}

/// In-memory rule store backed by defaults + database rules.
#[derive(Debug, Default)]
pub struct RuleStore {
    rules: Vec<CategoryRule>,
}

impl RuleStore {
    pub fn with_defaults() -> Self {
        Self {
            rules: vec![
                CategoryRule {
                    pattern: "github.com".into(),
                    pattern_type: RulePatternType::Domain,
                    category: ActivityCategory::Work,
                    priority: 10,
                },
                CategoryRule {
                    pattern: "stackoverflow.com".into(),
                    pattern_type: RulePatternType::Domain,
                    category: ActivityCategory::Learning,
                    priority: 10,
                },
                CategoryRule {
                    pattern: "youtube.com".into(),
                    pattern_type: RulePatternType::Domain,
                    category: ActivityCategory::Learning,
                    priority: 5,
                },
                CategoryRule {
                    pattern: "instagram.com".into(),
                    pattern_type: RulePatternType::Domain,
                    category: ActivityCategory::Distraction,
                    priority: 10,
                },
                CategoryRule {
                    pattern: "code".into(),
                    pattern_type: RulePatternType::AppName,
                    category: ActivityCategory::Work,
                    priority: 8,
                },
            ],
        }
    }

    pub fn add(&mut self, rule: CategoryRule) {
        self.rules.push(rule);
        self.sort_rules();
    }

    pub fn replace_all(&mut self, rules: Vec<CategoryRule>) {
        self.rules = rules;
        self.sort_rules();
    }

    pub fn merge(&mut self, rules: impl IntoIterator<Item = CategoryRule>) {
        self.rules.extend(rules);
        self.sort_rules();
    }

    pub fn rules(&self) -> &[CategoryRule] {
        &self.rules
    }

    fn sort_rules(&mut self) {
        self.rules.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then(b.pattern.len().cmp(&a.pattern.len()))
        });
    }
}
