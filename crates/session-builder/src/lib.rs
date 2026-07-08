//! Groups raw activity events into meaningful sessions.

mod builder;
mod config;
mod network;

pub use builder::{BuiltSession, NetworkObservation, SessionBuilder, TrackedAppLog};
pub use config::SessionBuilderConfig;
pub use network::network_stability_for_window;
