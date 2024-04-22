use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Mutex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeMap;
use serde_with::serde_as;

#[derive(Clone,Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    AtRisk,
    Unhealthy,
}

#[derive(Clone,Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HealthStatusDetails {
    pub current_status: HealthStatus,
    /**
    Decremented on each health check failure, retries stop when this hit's zero.
     */
    pub lives_remaining: u8 // TBD: proper value size
}

#[derive(Clone,Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HealthCheck {
    pub configuration: HealthCheckConfiguration,
    pub status_details: HealthStatusDetails
}

#[derive(Clone,Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct HealthCheckKey {
    pub(crate) port: u16,
}

impl Display for HealthCheckKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.port)
    }
}
// #[serde_as]
#[derive(Debug)]
pub struct HealthChecks {
    // #[serde_as(as = "HashMap<HealthCheckKey, HealthCheck>")]
    health_checks: Mutex<HashMap<HealthCheckKey, HealthCheck>>,
}

// // Let's just do the deserialize to keep everyone happy.... nvm!
//
// impl Deserialize for HealthChecks {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
//
//     }
//
//     fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error> where D: Deserializer<'de> {
//         todo!()
//     }
// }

impl Display for HealthChecks {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.health_checks)
    }
}

impl Serialize for HealthChecks {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let health_checks = self.health_checks.lock().unwrap().clone();
        let mut map = serializer.serialize_map(Some(health_checks.len()))?;
        for (k, v) in &health_checks {
            map.serialize_entry(&k.to_string(), &v)?;
        }
        map.end()
    }
}

impl Eq for HealthChecks {

}


impl PartialEq for HealthChecks {
    fn eq(&self, other: &Self) -> bool {
        let my_health_checks = self.health_checks.lock().unwrap();
        let your_health_checks = other.health_checks.lock().unwrap();

        if my_health_checks.len() != your_health_checks.len() {
            return false;
        }

        for my_record in my_health_checks.iter() {
            let my_key = my_record.0;
            let my_health_check = my_record.1;

            let your_health_check_option = your_health_checks.get(my_key);
            if your_health_check_option.is_none() {
                return false;
            }

            if my_health_check != your_health_check_option.unwrap() {
                return false
            }
        }

        return true
    }

    fn ne(&self, other: &Self) -> bool {
        return !self.eq(other)
    }
}

impl Clone for HealthChecks {
    fn clone(&self) -> Self {
        let actual_map: HashMap<HealthCheckKey, HealthCheck> =
            self.health_checks.lock().unwrap().clone();
        return Self {
            health_checks: Mutex::new(actual_map),
        };
    }
}



impl HealthChecks {
    pub fn new() -> Self {
        let actual_map: HashMap<HealthCheckKey, HealthCheck> = HashMap::new();
        return Self {
            health_checks: Mutex::new(actual_map),
        };
    }

    pub fn put_health_check(&mut self, health_check_key: HealthCheckKey, health_check: HealthCheck) {
        let mut health_checks = self.health_checks.lock().unwrap();
        health_checks.insert(health_check_key.clone(), health_check.clone());
    }

    pub fn get_health_check(&self, health_check_key: HealthCheckKey) -> Result<HealthCheck, ()>{
        let health_checks = self.health_checks.lock().unwrap();
        let health_check = health_checks.get(&health_check_key);
        if health_check.is_none() {
            return Err(())
        }

        return Ok(health_check.unwrap().clone())
    }
}

#[derive(Clone,Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HealthCheckConfiguration {
    pub health_check_port: u16,
    // ttl: u32

}
