
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

use crate::health_check::{HEALTH_CHECK_ACK_OPCODE, HEALTH_CHECK_SYN_OPCODE, NOOP_OPCODE};
use crate::health_check_network_broker::{HealthCheckNetworkBrokerMessage};
use crate::network::NetworkDetailsStore;

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
                network_details_store: &self.network_details_store
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



#[derive(Clone)]
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

fn health_check_ack_opcode_handler(context: HealthCheckHandlerContext, params: OpcodeHandlerParams) {
    println!("TODO: implement health_check_ack_opcode_handler")
}

fn health_check_noop_opcode_handler(context: HealthCheckHandlerContext, params: OpcodeHandlerParams) {
    println!("TODO: implement health_check_noop_opcode_handler");
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