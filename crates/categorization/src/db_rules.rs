use netchronicle_db::CategoryRuleRow;

use crate::rules::{CategoryRule, RulePatternType, RuleStore};

/// Build a rule store from database rows merged with built-in defaults.
pub fn rule_store_from_db(rows: &[CategoryRuleRow]) -> RuleStore {
    let mut store = RuleStore::with_defaults();
    for row in rows {
        if let Some(rule) = rule_from_row(row) {
            store.add(rule);
        }
    }
    store
}

pub fn rule_from_row(row: &CategoryRuleRow) -> Option<CategoryRule> {
    let pattern_type = match row.pattern_type.as_str() {
        "domain" => RulePatternType::Domain,
        "url_prefix" | "url" => RulePatternType::UrlPrefix,
        "app_name" | "app" => RulePatternType::AppName,
        _ => return None,
    };

    Some(CategoryRule {
        pattern: row.pattern.to_lowercase(),
        pattern_type,
        category: netchronicle_db::parse_category(&row.category),
        priority: row.priority,
    })
}
