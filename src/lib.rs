//! ITCy ratatui status library.

pub mod health;
pub mod ui;

pub use health::{fetch_health, interpret_health, HealthStatus};
