//! Rule-based and learnable categorization (work, learning, distraction, etc.).

mod classifier;
mod rules;

pub use classifier::Categorizer;
pub use rules::{CategoryRule, RuleStore};
