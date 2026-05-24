//! Groups raw logs into sessions (time windows, apps, network context).

mod builder;
mod config;

pub use builder::SessionBuilder;
pub use config::SessionBuilderConfig;
