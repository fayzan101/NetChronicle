use netchronicle_common::ActivityCategory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRule {
    pub pattern: String,
    pub pattern_type: RulePatternType,
    pub category: ActivityCategory,
    pub priority: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RulePatternType {
    Domain,
    UrlPrefix,
    AppName,
}

/// In-memory rule store; later backed by `category_rules` table.
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
            ],
        }
    }

    pub fn rules(&self) -> &[CategoryRule] {
        &self.rules
    }
}
