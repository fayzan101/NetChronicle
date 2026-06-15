use netchronicle_common::ActivityCategory;

use crate::rules::RulePatternType;
use crate::RuleStore;
pub struct Categorizer {
    rules: RuleStore,
}
impl Categorizer {
    pub fn new(rules: RuleStore) -> Self {
        Self { rules }
    }
    pub fn classify_domain(&self, domain: &str) -> ActivityCategory {
        let domain = domain.to_lowercase();
        for rule in self.rules.rules() {
            if domain.contains(&rule.pattern) {
                return rule.category;
            }
        }
        ActivityCategory::Unknown
    }

    pub fn classify_app(&self, app_name: &str) -> ActivityCategory {
        let app = app_name.to_lowercase();
        for rule in self.rules.rules() {
            if matches!(rule.pattern_type, RulePatternType::AppName)
                && app.contains(&rule.pattern.to_lowercase())
            {
                return rule.category;
            }
        }

        if app.contains("code") || app.contains("devenv") || app.contains("idea") {
            return ActivityCategory::Work;
        }
        if app.contains("chrome") || app.contains("firefox") || app.contains("msedge") {
            return ActivityCategory::Neutral;
        }

        ActivityCategory::Unknown
    }

    pub fn classify_activity(
        &self,
        app_name: &str,
        domain: Option<&str>,
    ) -> ActivityCategory {
        if let Some(domain) = domain {
            let by_domain = self.classify_domain(domain);
            if by_domain != ActivityCategory::Unknown {
                return by_domain;
            }
        }
        self.classify_app(app_name)
    }
}
