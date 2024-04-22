use std::net::IpAddr;
use serde::{Deserialize, Serialize};
use crate::health_check_model::HealthChecks;

#[derive(Clone,Debug, Eq, PartialEq, Serialize)]
pub struct NetworkDetails {
    pub addr: IpAddr,
    pub health_checks: HealthChecks,
}
