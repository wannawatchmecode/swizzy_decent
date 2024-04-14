use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crate::health_check::{DeserializePacket, HEALTH_CHECK_PACKET_SIZE, HealthCheckPacket, SerializePacket};

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
    request_receiver: Receiver<HealthCheckNetworkBrokerMessage>,
    response_sender: Sender<HealthCheckNetworkBrokerMessage>
}

impl HealthCheckNetworkBroker {
    pub fn new(socket_addr: SocketAddr,
               request_receiver: Receiver<HealthCheckNetworkBrokerMessage>,
               response_sender: Sender<HealthCheckNetworkBrokerMessage>) -> HealthCheckNetworkBroker {
        HealthCheckNetworkBroker {
            socket_addr,
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
}

fn health_check_receiver(socket: UdpSocket, response_sender: Sender<HealthCheckNetworkBrokerMessage>) -> std::io::Result<()> {
    {
        println!("Health Check message received");
        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; HEALTH_CHECK_PACKET_SIZE+1];
        let (amt, src) = socket.recv_from(&mut buf)?;
        let mut buf = &buf[..HEALTH_CHECK_PACKET_SIZE];
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

        let mut buf = &mut [0;HEALTH_CHECK_PACKET_SIZE];
        let raw = request_object.serialize();
        for i in 0..HEALTH_CHECK_PACKET_SIZE {
            buf[i] = raw[i];
        }

        let dst = message.remote_addr;
        let amt = socket.send_to(buf, &dst)?;
        println!("Health check message sent")
    } // the socket is closed here
    Ok(())
}