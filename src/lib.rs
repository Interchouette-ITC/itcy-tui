//! ITCy ratatui status library.

pub mod commands;
pub mod health;
pub mod status;
pub mod ui;

pub use health::{fetch_health, interpret_health, HealthStatus};
pub use status::{fetch_status, RuntimeStatus};
pub use ui::{StatusModel, ViewMode};
