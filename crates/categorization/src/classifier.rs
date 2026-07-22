use netchronicle_common::ActivityCategory;

use crate::rules::{CategoryRule, RulePatternType, RuleStore};

pub struct Categorizer {
    rules: RuleStore,
}

impl Categorizer {
    pub fn new(rules: RuleStore) -> Self {
        Self { rules }
    }

    pub fn classify_domain(&self, domain: &str) -> ActivityCategory {
        let domain = domain.to_lowercase();
        self.best_match(RulePatternType::Domain, |rule| {
            domain.contains(&rule.pattern)
        })
        .unwrap_or(ActivityCategory::Unknown)
    }

    pub fn classify_url(&self, url: &str) -> ActivityCategory {
        let url = url.to_lowercase();
        self.best_match(RulePatternType::UrlPrefix, |rule| {
            url.starts_with(&rule.pattern) || url.contains(&rule.pattern)
        })
        .unwrap_or(ActivityCategory::Unknown)
    }

    pub fn classify_app(&self, app_name: &str) -> ActivityCategory {
        let app = app_name.to_lowercase();
        if let Some(category) =
            self.best_match(RulePatternType::AppName, |rule| app.contains(&rule.pattern))
        {
            return category;
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
        url: Option<&str>,
        domain: Option<&str>,
    ) -> ActivityCategory {
        if let Some(url) = url {
            let by_url = self.classify_url(url);
            if by_url != ActivityCategory::Unknown {
                return by_url;
            }
        }
        if let Some(domain) = domain {
            let by_domain = self.classify_domain(domain);
            if by_domain != ActivityCategory::Unknown {
                return by_domain;
            }
        }
        self.classify_app(app_name)
    }

    fn best_match<F>(&self, pattern_type: RulePatternType, matches: F) -> Option<ActivityCategory>
    where
        F: Fn(&CategoryRule) -> bool,
    {
        self.rules
            .rules()
            .iter()
            .filter(|rule| rule.pattern_type == pattern_type && matches(rule))
            .max_by_key(|rule| (rule.priority, rule.pattern.len()))
            .map(|rule| rule.category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{CategoryRule, RulePatternType};

    #[test]
    fn higher_priority_wins() {
        let mut store = RuleStore::default();
        store.add(CategoryRule {
            pattern: "github.com".into(),
            pattern_type: RulePatternType::Domain,
            category: ActivityCategory::Distraction,
            priority: 1,
        });
        store.add(CategoryRule {
            pattern: "github.com".into(),
            pattern_type: RulePatternType::Domain,
            category: ActivityCategory::Work,
            priority: 20,
        });

        let categorizer = Categorizer::new(store);
        assert_eq!(
            categorizer.classify_domain("github.com"),
            ActivityCategory::Work
        );
    }

    #[test]
    fn longer_pattern_wins_at_same_priority() {
        let mut store = RuleStore::default();
        store.add(CategoryRule {
            pattern: "git".into(),
            pattern_type: RulePatternType::Domain,
            category: ActivityCategory::Neutral,
            priority: 10,
        });
        store.add(CategoryRule {
            pattern: "github.com".into(),
            pattern_type: RulePatternType::Domain,
            category: ActivityCategory::Work,
            priority: 10,
        });

        let categorizer = Categorizer::new(store);
        assert_eq!(
            categorizer.classify_domain("docs.github.com"),
            ActivityCategory::Work
        );
    }
}
