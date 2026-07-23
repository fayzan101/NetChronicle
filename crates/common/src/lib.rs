//! Shared domain types used across agent, processing, API, and DB layers.

mod auth;
mod error;
mod event;
mod session;

pub use auth::{
    api_key_prefix, generate_api_key, generate_bearer_token, hash_secret, hash_token, verify_secret,
};
pub use error::{Error, Result};
pub use event::{
    ActivityCategory, AppActivityEvent, NetworkStability, RawEvent, WebsiteVisitEvent,
};
pub use session::{DailySummary, Session, SessionDraft};
