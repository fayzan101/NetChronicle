//! Rule-based and learnable categorization (work, learning, distraction, etc.).

mod classifier;
mod db_rules;
mod rules;

pub use classifier::Categorizer;
pub use db_rules::{rule_from_row, rule_store_from_db};
pub use rules::{CategoryRule, RulePatternType, RuleStore};
