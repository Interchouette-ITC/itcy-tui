//! Smoke: health interpret + UI render (also covered in unit tests).

use itcy_tui::{interpret_health, HealthStatus};

#[test]
fn health_contract_matches_product() {
    assert_eq!(interpret_health(200, "ok"), HealthStatus::Ok);
}
