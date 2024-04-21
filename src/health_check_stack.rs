use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use crate::health_check_network_broker::{HealthCheckNetworkBroker, HealthCheckNetworkBrokerMessage};
use crate::health_check_network_handlers::HealthCheckNetworkBrokerMessageListener;
use crate::network::NetworkDetailsStore;

pub struct HealthCheckStack<'a> {
    /**
    Clone of inner network broker request_sender channel.
     */
    pub request_sender: Sender<HealthCheckNetworkBrokerMessage>, // Temporary maybe
    pub network_broker: HealthCheckNetworkBroker, // todo: make private
    pub health_check_network_broker_message_listener: HealthCheckNetworkBrokerMessageListener<'a>, // todo: make private
    // pub network_details_store: &'a NetworkDetailsStore
}

impl HealthCheckStack<'_> {

    pub fn new(network_broker: HealthCheckNetworkBroker,
                   health_check_network_broker_message_listener: HealthCheckNetworkBrokerMessageListener,
                   ) -> HealthCheckStack {

        return HealthCheckStack {
            request_sender: network_broker.request_sender.clone(),
            network_broker,
            health_check_network_broker_message_listener,
            // network_details_store
        }
    }

    pub fn run(self) {
        let health_check_network_broker_message_listener = self.health_check_network_broker_message_listener;
        let network_broker = self.network_broker;
        let listener_handler = thread::spawn(move ||{
            health_check_network_broker_message_listener.run()
        });

        let broker_handler = thread::spawn(move || {
            network_broker.run()
        });

        broker_handler.join().expect("Joined network broker in HealthCheckStack");
        listener_handler.join().expect("Joined listener in HealthCheckStack");
    }
}

pub fn build_health_check_stack(receiver_addr: SocketAddr, network_details_store: &NetworkDetailsStore) -> HealthCheckStack {
    let (request_sender, request_receiver) = mpsc::channel();
    let (response_sender, response_receiver) = mpsc::channel();

    let network_broker = HealthCheckNetworkBroker::new(receiver_addr, request_sender.clone(), request_receiver, response_sender);
    // let network_details_store: NetworkDetailsStore = NetworkDetailsStore::new();
    let health_check_network_broker_message_listener = HealthCheckNetworkBrokerMessageListener::new(response_receiver, request_sender.clone(), network_details_store);

    return HealthCheckStack::new(
        network_broker,
        health_check_network_broker_message_listener,
        // &network_details_store
    );
}