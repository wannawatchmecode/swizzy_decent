use std::fmt::Debug;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use log::error;


use crate::health_check::{DeserializePacket, HEALTH_CHECK_PACKET_SIZE, HealthCheckPacket, SerializePacket};
use crate::health_check_network_handlers::HealthCheckNetworkBrokerMessageListener;
use crate::network::{NetworkDetailsStore};

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
                let receiver_socket_res = receiver_socket.try_clone();
                if receiver_socket_res.is_err() {
                    error!("Error cloning socket in network broker run method");
                    continue;
                }
                let receiver_socket = receiver_socket_res.unwrap();
                let response_sender = response_sender.clone();
                _ = health_check_receiver(receiver_socket, response_sender);
            }
        });

        let sender_socket = socket.try_clone().expect("Sender socket cloned");

        let request_receiver = self.request_receiver;
        let send_handle = thread::spawn(move || {
            loop {
                let next_request_res = request_receiver.recv(); //.expect("HealthCheckNetworkBrokerMessage received from request_receiver"); // TODO: uncomment after perf testing
                if next_request_res.is_err() {
                    error!("Error with sender thread receiver");
                    continue;
                }
                let next_request = next_request_res.unwrap();
                let sender_socket_res = sender_socket.try_clone();
                if sender_socket_res.is_err() {
                    error!("Error cloning sender socket");
                    continue;
                }
                let sender_socket = sender_socket_res.unwrap();
                _ = health_check_sender(sender_socket, next_request)
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
    ) -> HealthCheckStack {

        return HealthCheckStack {
            request_sender: network_broker.request_sender.clone(),
            network_broker,
            health_check_network_broker_message_listener,
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