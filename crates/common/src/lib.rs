//! Shared domain types used across agent, processing, API, and DB layers.

mod error;
mod event;
mod session;

pub use error::{Error, Result};
pub use event::{ActivityCategory, AppActivityEvent, NetworkStability, RawEvent, WebsiteVisitEvent};
pub use session::{DailySummary, Session, SessionDraft};
