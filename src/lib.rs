// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! `ITCy` ratatui status library.

pub mod artefact;
pub mod commands;
pub mod github;
pub mod health;
pub mod inject;
pub mod status;
pub mod ui;

pub use health::{fetch_health, interpret_health, HealthStatus};
pub use status::{fetch_status, RuntimeStatus};
pub use ui::{StatusModel, ViewMode};
