use std::net::IpAddr;
use crate::health_check_model::HealthChecks;

#[derive(Clone,Debug, Eq, PartialEq)]
pub struct NetworkDetails {
    pub addr: IpAddr,
    pub health_checks: HealthChecks,
}
