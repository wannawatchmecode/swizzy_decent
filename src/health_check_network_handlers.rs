
// Handlers
// Listen to a receiver channel
// Process broker message
// handle
// Syn
// Send ack
// Ack request
// TODO: update the network table
// NOOP - log unexpected message

use std::collections::{HashMap};
use std::sync::mpsc::{Receiver, Sender};
use log::{debug, info, warn};

use crate::health_check::{HEALTH_CHECK_ACK_OPCODE, HEALTH_CHECK_SYN_OPCODE, NOOP_OPCODE};
use crate::health_check_network_broker::{HealthCheckNetworkBrokerMessage};
use crate::health_check_model::{HealthCheck, HealthCheckConfiguration, HealthCheckKey, HealthChecks, HealthStatus, HealthStatusDetails};
use crate::network::{NETWORK_DETAILS_STORE, NetworkDetailsStore};
use crate::network_models::NetworkDetails;

pub struct HealthCheckNetworkBrokerMessageListener {
    health_check_handler_map: HashMap<u8, fn(context: HealthCheckHandlerContext, params: OpcodeHandlerParams)>,
    /**
    Receiver from the network broker.
     */
    network_broker_receiver: Receiver<HealthCheckNetworkBrokerMessage>,
    /**
    Sender to the network broker.
     */
    network_broker_sender: Sender<HealthCheckNetworkBrokerMessage>,

    /**

    */
    network_details_store: NetworkDetailsStore
}

impl HealthCheckNetworkBrokerMessageListener {
    pub fn new(network_broker_receiver: Receiver<HealthCheckNetworkBrokerMessage>,
               network_broker_sender: Sender<HealthCheckNetworkBrokerMessage>,
    network_details_store: NetworkDetailsStore) -> HealthCheckNetworkBrokerMessageListener {
        HealthCheckNetworkBrokerMessageListener {
            health_check_handler_map: get_health_check_handler_map(),
            network_broker_receiver,
            network_broker_sender,
            network_details_store
        }
    }

    pub fn run(self) {
        // let receiver_handle = thread::spawn(move || {
        loop  {
            let next_message = self.network_broker_receiver.recv().expect("HealthCheckNetworkBrokerMessageListener message received ");
            let handler_fn = self.health_check_handler_map
                .get(&next_message.payload.header)
                .expect("Handler method to be found from message payload header op code");

            let handler_props = OpcodeHandlerParams {
                message: next_message,
                sender: self.network_broker_sender.clone()
            };
            let context = HealthCheckHandlerContext {
                // network_details_store: &self.network_details_store
                network_details_store: &NETWORK_DETAILS_STORE
            };

            handler_fn(context, handler_props);
        }
    }

    // pub fn handle_message(self, message: HealthCheckNetworkBrokerMessage) {
    //
    //     let handler_fn = self.health_check_handler_map
    //         .get(&message.payload.header)
    //         .expect("Handler method to be found from message payload header op code");
    //
    //     handler_fn(message);
    // }

}



#[derive(Clone, Debug)]
pub struct OpcodeHandlerParams {
    message: HealthCheckNetworkBrokerMessage,
    sender: Sender<HealthCheckNetworkBrokerMessage>
}

fn health_check_syn_opcode_handler(context: HealthCheckHandlerContext, params: OpcodeHandlerParams) {
    let mut response_object = params.message.clone();
    response_object.payload.header = HEALTH_CHECK_ACK_OPCODE;
    params.sender.send(response_object)
        .expect("Health Check ack response sent to message broker");
}

const MAX_LIVES_REMAINING:u8 = 3;

fn health_check_ack_opcode_handler(context: HealthCheckHandlerContext, params: OpcodeHandlerParams) {
    info!("Ack received from {}", params.message.remote_addr);
    debug!("Params: {:?}", params);
    // Not sure this is necessary at this point, mainly if we would want to check if a record already exists or not
    let mut existing_record_retrieve_result = context.network_details_store.get_network_details_by_ip(&params.message.remote_addr.ip().clone());
    // I think what I actually want to do is just add or update a record in the NetworkDetailsStore, so good to have the existing record for updating
    let mut new_record;

    let health_check_key = HealthCheckKey {
        port: params.message.remote_addr.port()
    };

    if !existing_record_retrieve_result.is_ok() {
        info!("record not found in network details store, will create a new one");
        let mut health_checks = HealthChecks::new();


        let new_health_check = HealthCheck {
            status_details: HealthStatusDetails {
                current_status: HealthStatus::Healthy,
                lives_remaining: MAX_LIVES_REMAINING // TODO: Probably make configurable
            },
            configuration: HealthCheckConfiguration {
                health_check_port: params.message.remote_addr.port(),
            }
        };

        health_checks.put_health_check(health_check_key, new_health_check);
        new_record = NetworkDetails {
            addr: params.message.remote_addr.ip().clone(),
            health_checks: health_checks
        };
        context.network_details_store.put_network_details(&new_record);
    } else {
        let mut old_record = existing_record_retrieve_result.unwrap();
        let health_checks = old_record.clone().health_checks;
        if health_checks.get_health_check(health_check_key.clone()).is_ok() {
            let mut health_check = health_checks.get_health_check(health_check_key.clone()).unwrap();
            let old_lives_remaining: u8 = health_check.status_details.lives_remaining;
            if old_lives_remaining < MAX_LIVES_REMAINING {
                health_check.status_details.lives_remaining = old_lives_remaining + 1;
            }
            old_record.health_checks.put_health_check(health_check_key, health_check);
            new_record = old_record.clone();
            // context.network_details_store.put_network_details(&old_record);
        } else {
            let mut health_checks = health_checks;


            let new_health_check = HealthCheck {
                status_details: HealthStatusDetails {
                    current_status: HealthStatus::Healthy,
                    lives_remaining: MAX_LIVES_REMAINING // TODO: Probably make configurable
                },
                configuration: HealthCheckConfiguration {
                    health_check_port: params.message.remote_addr.port(),
                }
            };

            health_checks.put_health_check(health_check_key, new_health_check);
            new_record = NetworkDetails {
                addr: params.message.remote_addr.ip().clone(),
                health_checks
            };
        }
        // One thing I'm thinking of is how to handle the health changes, would be nice to introduce a
        // configurable policy that determines how to refresh the "lives" for the health status
        // For now I think I'll just set lives to max on a single successful health check
        // But was thinking about scenarios where you might want to "refill" lives with a different strategy
        // One option could be to add a live per successful health check, to give a SMA like value for the
        // Health status. After typing this out I will go with the incrementing policy

        // let old_lives_remaining: u8 = new_record.health_checks.status_details.lives_remaining;
        // if old_lives_remaining < MAX_LIVES_REMAINING {
        //     new_record.health_check.status_details.lives_remaining = old_lives_remaining + 1;
        // }
    }

    context.network_details_store.put_network_details(&new_record);
    info!("Updated network details {:?}", context.network_details_store);
}

fn health_check_noop_opcode_handler(context: HealthCheckHandlerContext, params: OpcodeHandlerParams) {
    warn!("TODO: implement health_check_noop_opcode_handler");
}

struct HealthCheckHandlerContext<'a> {
    network_details_store: &'a NetworkDetailsStore
}

// health_check_handler_map: HashMap<u8, fn(context: &HealthCheckHandlerContext, params: OpcodeHandlerParams)>,

pub fn get_health_check_handler_map() -> HashMap<u8, fn(context: HealthCheckHandlerContext, params: OpcodeHandlerParams)> {
    let mut map: HashMap<u8, fn(context: HealthCheckHandlerContext, params: OpcodeHandlerParams)> = HashMap::new();
    map.insert(NOOP_OPCODE, health_check_noop_opcode_handler);
    map.insert(HEALTH_CHECK_SYN_OPCODE, health_check_syn_opcode_handler);
    map.insert(HEALTH_CHECK_ACK_OPCODE, health_check_ack_opcode_handler);
    return map;
    // from((NOOP_OPCODE, health_check_noop_opcode_handler, HEALTH_CHECK_SYN_OPCODE, health_check_syn_opcode_handler, HEALTH_CHECK_ACK_OPCODE, health_check_ack_opcode_handler),);
}

#[cfg(test)]
mod health_check_tests {
    use crate::health_check::{HEALTH_CHECK_ACK_OPCODE, HEALTH_CHECK_SYN_OPCODE, NOOP_OPCODE};
    use crate::health_check_network_handlers::{get_health_check_handler_map};

    #[test]
    fn health_check_handler_map_contains_handlers() {
        let handler_map = get_health_check_handler_map();
        handler_map.get(&HEALTH_CHECK_SYN_OPCODE).unwrap();
        handler_map.get(&HEALTH_CHECK_ACK_OPCODE).unwrap();
        handler_map.get(&NOOP_OPCODE).unwrap();
        // let invalid_key: u8 = 10;
        // handler_map.get(&invalid_key).unwrap();
    }
}