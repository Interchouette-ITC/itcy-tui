// Copyright (c) 2026 Interchouette-ITC
// SPDX-License-Identifier: BUSL-1.1

//! Smoke: health interpret + UI render (also covered in unit tests).

use itcy_tui::{interpret_health, HealthStatus};

#[test]
fn health_contract_matches_product() {
    assert_eq!(interpret_health(200, "ok"), HealthStatus::Ok);
}
