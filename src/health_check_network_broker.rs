use std::fmt::Debug;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use log::info;


use crate::health_check::{DeserializePacket, HEALTH_CHECK_PACKET_SIZE, HealthCheckPacket, SerializePacket};
use crate::health_check_network_handlers::HealthCheckNetworkBrokerMessageListener;
use crate::network::{IP, NetworkDetailsStore, RECEIVER_PORT, SENDER_PORT};

#[derive(Clone, Debug)]
pub struct HealthCheckNetworkBrokerMessage {
    pub payload: HealthCheckPacket,
    /*
        SocketAddress of remote host where the payload was sent to or received from.
    */
    pub remote_addr: SocketAddr
}

pub struct HealthCheckNetworkBroker {
    socket_addr: SocketAddr,
    pub request_sender: Sender<HealthCheckNetworkBrokerMessage>,
    request_receiver: Receiver<HealthCheckNetworkBrokerMessage>,
    response_sender: Sender<HealthCheckNetworkBrokerMessage>
}

impl HealthCheckNetworkBroker {
    pub fn new(socket_addr: SocketAddr,
               request_sender: Sender<HealthCheckNetworkBrokerMessage>,
               request_receiver: Receiver<HealthCheckNetworkBrokerMessage>,
               response_sender: Sender<HealthCheckNetworkBrokerMessage>) -> HealthCheckNetworkBroker {
        HealthCheckNetworkBroker {
            socket_addr,
            request_sender,
            request_receiver,
            response_sender
        }
    }

    pub fn run(self) {
        println!("Starting run process for HealthCheckNetworkBroker");
        let socket = UdpSocket::bind(self.socket_addr).expect("Socket bound");
        let receiver_socket = socket.try_clone().expect("Receiver socket cloned");

        let response_sender = self.response_sender;
        let receiver_handle = thread::spawn(move || {
            loop  {
                let receiver_socket = receiver_socket.try_clone().expect("Cloned receiver socket");
                let response_sender = response_sender.clone();
                health_check_receiver(receiver_socket, response_sender).expect("Health check receiver succeeded");
            }
        });

        let sender_socket = socket.try_clone().expect("Sender socket cloned");

        let request_receiver = self.request_receiver;
        let send_handle = thread::spawn(move || {
            loop {
                let next_request = request_receiver.recv().expect("HealthCheckNetworkBrokerMessage received from request_receiver"); // TODO: uncomment after perf testing
                // let res = request_receiver.recv_timeout(Duration::new(1, 0));
                // if res.clone().is_err() {
                //     break;
                // }
                // let next_request = res.unwrap();
                let sender_socket = sender_socket.try_clone().expect("Cloned");
                    health_check_sender(sender_socket, next_request).expect("Sent HealthCheckNetworkBrokerMessage to remote addr");
                // sleep(Duration::new(0,1));
            }
        });
        println!("Started threads for HealthCheckNetworkBroker");
        send_handle.join().unwrap();
        receiver_handle.join().unwrap();
        println!("HealthCheckNetworkBroker run complete");
    }

    pub fn get_request_sender(self) -> Sender<HealthCheckNetworkBrokerMessage>{
        return self.request_sender.clone()
    }
}

fn health_check_receiver(socket: UdpSocket, response_sender: Sender<HealthCheckNetworkBrokerMessage>) -> std::io::Result<()> {
    {
        println!("Health Check receiver waiting for messages");
        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; HEALTH_CHECK_PACKET_SIZE+1];
        let (_amt, src) = socket.recv_from(&mut buf)?;
        let buf = &buf[..HEALTH_CHECK_PACKET_SIZE];
        let buf_vec = buf.to_vec();
        let health_check_packet = HealthCheckPacket::deserialize(buf_vec);

        println!("Received: {:?}", health_check_packet);
        response_sender.send(HealthCheckNetworkBrokerMessage {
            payload: health_check_packet,
            remote_addr: src
        }).expect("HealthCheckBroken receiver forwards received messages to response_sender channel");
        println!("Response sent to response_sender channel")
    } // the socket is closed here
    Ok(())
}

fn health_check_sender(socket: UdpSocket, message: HealthCheckNetworkBrokerMessage) -> std::io::Result<()> {
    {
        println!("Health check sender invoked");
        let request_object = message.payload;

        let buf = &mut [0;HEALTH_CHECK_PACKET_SIZE];
        let raw = request_object.serialize();
        for i in 0..HEALTH_CHECK_PACKET_SIZE {
            buf[i] = raw[i];
        }

        let dst = message.remote_addr;
        let _amt = socket.send_to(buf, &dst)?;
        println!("Health check message sent")
    } // the socket is closed here
    Ok(())
}

pub struct HealthCheckStack {
    /**
        Clone of inner network broker request_sender channel.
    */
    pub request_sender: Sender<HealthCheckNetworkBrokerMessage>, // Temporary maybe
    pub network_broker: HealthCheckNetworkBroker, // todo: make private
    pub health_check_network_broker_message_listener: HealthCheckNetworkBrokerMessageListener, // todo: make private
    // network_details_store: &'static NetworkDetailsStore
}

impl HealthCheckStack {

    pub fn new(network_broker: HealthCheckNetworkBroker,
               health_check_network_broker_message_listener: HealthCheckNetworkBrokerMessageListener,
               // network_details_store: NetworkDetailsStore
    ) -> HealthCheckStack {

        return HealthCheckStack {
            request_sender: network_broker.request_sender.clone(),
            network_broker,
            health_check_network_broker_message_listener,
            // network_details_store
        }
    }

    pub fn run(self) {
        let listener_handler = thread::spawn(move || {
            self.health_check_network_broker_message_listener.run();
        });

        let broker_handler = thread::spawn(move || {
            self.network_broker.run();
        });

        broker_handler.join().expect("Joined network broker in HealthCheckStack");
        listener_handler.join().expect("Joined listener in HealthCheckStack");
    }
}

pub struct HealthCheckFactory {

}

// impl HealthCheckFactory {
//     pub fn build(receiver_addr: SocketAddr) -> HealthCheckStack {
//         let (message_sender, message_receiver) = mpsc::channel();
//         let (consumer_sender, consumer_receiver) = mpsc::channel();
//
//         // let test_message_sender = message_sender.clone();
//         //     let _message_sender = message_sender.clone();
//         //     let receiver_addr = SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT);
//             let network_broker = HealthCheckNetworkBroker::new(receiver_addr, message_sender.clone(), message_receiver, consumer_sender);
//             println!("Created message_broker_2");
//             println!("Attempting to run message_broker_2");
//             // message_broker_2.run();
//             println!("message_broker_2 finished running");
//         // let network_broker_for_listener = &network_broker;
//         let health_check_network_broker_message_listener = HealthCheckNetworkBrokerMessageListener::new(consumer_receiver, message_sender.clone());
//         // let network_broker = network_broker;
//         return HealthCheckStack {
//             network_broker,
//             health_check_network_broker_message_listener
//         }
//     }
// }

pub fn build_health_check_stack(receiver_addr: SocketAddr) -> HealthCheckStack {
    let (request_sender, request_receiver) = mpsc::channel();
    let (response_sender, response_receiver) = mpsc::channel();

    let network_broker = HealthCheckNetworkBroker::new(receiver_addr, request_sender.clone(), request_receiver, response_sender);
    // let network_details_store = NetworkDetailsStore::new();
    let health_check_network_broker_message_listener = HealthCheckNetworkBrokerMessageListener::new(response_receiver, request_sender.clone(), NetworkDetailsStore::new());

    return HealthCheckStack::new(
        network_broker,
        health_check_network_broker_message_listener
    )
}