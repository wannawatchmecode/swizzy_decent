use std::net::{IpAddr, SocketAddr};
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::health_check::{DeserializePacket, HealthCheckPacket, SerializePacket};
use crate::health_check_network_broker::{HealthCheckNetworkBroker, HealthCheckNetworkBrokerMessage};
use crate::health_check_network_handlers::HealthCheckNetworkBrokerMessageListener;
use crate::network::{health_check_receiver, health_check_sender, IP, RECEIVER_PORT, SENDER_PORT};

mod health_check;
mod network;
mod health_check_network_broker;
mod health_check_network_handlers;

fn main() {
    println!("HealthCheckBroker with receiver handler load example main");

    let message_sender_1_handle = thread::spawn(move || {
        let (message_sender1, message_receiver1) = mpsc::channel();
        let sender_addr = SocketAddr::new(IpAddr::V4(IP), SENDER_PORT);
        let message_broker_1 = HealthCheckNetworkBroker::new(sender_addr, message_receiver1, message_sender1);
        println!("Created message_broker_1");
        println!("Attempting to run message_broker_1");
        message_broker_1.run();
        println!("message_broker_1 finished running");
    });

    let (message_sender2, message_receiver2) = mpsc::channel();
    let (consumer_sender2, consumer_receiver2) = mpsc::channel();

    let test_message_sender = message_sender2.clone();
    let message_sender_2_handle = thread::spawn(move || {
        let message_sender = message_sender2.clone();
        let receiver_addr = SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT);
        let message_broker_2 = HealthCheckNetworkBroker::new(receiver_addr, message_receiver2, consumer_sender2);
        println!("Created message_broker_2");
        println!("Attempting to run message_broker_2");
        message_broker_2.run();
        println!("message_broker_2 finished running");
    });
    let message_sender2 = test_message_sender.clone();

    let receiver_handle = thread::spawn(move || {
        let broker_message_listener2 = HealthCheckNetworkBrokerMessageListener::new(consumer_receiver2, message_sender2);
        broker_message_listener2.run();
    });

    // Testing
    println!("Performing message send test");

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("{:?}", since_the_epoch);

    for i in 0..1000 {
        let message = HealthCheckNetworkBrokerMessage {
            payload: HealthCheckPacket {
                header: 1,
                nonce: [2,3,6,1,7,3,1,3,8,9,3,2,6,3,7,3]
            },
            remote_addr: SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT)
        };

        // println!("Sending message");
        test_message_sender.send(message).expect("Message sent");
        // println!("Message Sent");
    }

    let end = SystemTime::now();
    let end_since_the_epoch = end
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("Start: [{:?}] End: [{:?}] TotalDuration: [{:?}]", since_the_epoch, end_since_the_epoch, end_since_the_epoch-since_the_epoch);
    message_sender_2_handle.join().expect("message_sender_2_handle joined");

    // let message_from_broker = consumer_receiver2.recv().expect("Message to be received");
    // println!("Received message from broker: {:?}", message_from_broker);
    println!("Waiting for threads to complete");
    receiver_handle.join().expect("Receiver_handle joined");
    message_sender_1_handle.join().expect("message_sender_1_handle joined");
    // message_sender_2_handle.join().expect("message_sender_2_handle joined");
    println!("Application done running, exiting");
}

fn main_with_receiver_handler() {
    println!("HealthCheckBroker with receiver handler example main");

    let message_sender_1_handle = thread::spawn(move || {
        let (message_sender1, message_receiver1) = mpsc::channel();
        let sender_addr = SocketAddr::new(IpAddr::V4(IP), SENDER_PORT);
        let message_broker_1 = HealthCheckNetworkBroker::new(sender_addr, message_receiver1, message_sender1);
        println!("Created message_broker_1");
        println!("Attempting to run message_broker_1");
        message_broker_1.run();
        println!("message_broker_1 finished running");
    });

    let (message_sender2, message_receiver2) = mpsc::channel();
    let (consumer_sender2, consumer_receiver2) = mpsc::channel();

    let test_message_sender = message_sender2.clone();
    let message_sender_2_handle = thread::spawn(move || {
        let message_sender = message_sender2.clone();
        let receiver_addr = SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT);
        let message_broker_2 = HealthCheckNetworkBroker::new(receiver_addr, message_receiver2, consumer_sender2);
        println!("Created message_broker_2");
        println!("Attempting to run message_broker_2");
        message_broker_2.run();
        println!("message_broker_2 finished running");
    });
    let message_sender2 = test_message_sender.clone();

    let receiver_handle = thread::spawn(move || {
        let broker_message_listener2 = HealthCheckNetworkBrokerMessageListener::new(consumer_receiver2, message_sender2);
        broker_message_listener2.run();
    });

    // Testing
    println!("Performing message send test");
    let message = HealthCheckNetworkBrokerMessage {
        payload: HealthCheckPacket {
            header: 1,
            nonce: [2,3,6,1,7,3,1,3,8,9,3,2,6,3,7,3]
        },
        remote_addr: SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT)
    };
    println!("Sending message");
    test_message_sender.send(message).expect("Message sent");
    println!("Message Sent");
    // let message_from_broker = consumer_receiver2.recv().expect("Message to be received");
    // println!("Received message from broker: {:?}", message_from_broker);
    println!("Waiting for threads to complete");
    receiver_handle.join().expect("Receiver_handle joined");
    message_sender_1_handle.join().expect("message_sender_1_handle joined");
    message_sender_2_handle.join().expect("message_sender_2_handle joined");
    println!("Application done running, exiting");
}

fn main_health_check_broker_example() {
    println!("HealthCheckBroker example main");

    let message_sender_1_handle = thread::spawn(move || {
        let (message_sender1, message_receiver1) = mpsc::channel();
        let sender_addr = SocketAddr::new(IpAddr::V4(IP), SENDER_PORT);
        let message_broker_1 = HealthCheckNetworkBroker::new(sender_addr, message_receiver1, message_sender1);
        println!("Created message_broker_1");
        println!("Attempting to run message_broker_1");
        message_broker_1.run();
        println!("message_broker_1 finished running");
    });

    let (message_sender2, message_receiver2) = mpsc::channel();
    let (consumer_sender2, consumer_receiver2) = mpsc::channel();

let test_message_sender = message_sender2.clone();
    let message_sender_2_handle = thread::spawn(move || {
        let message_sender = message_sender2.clone();
        let receiver_addr = SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT);
        let message_broker_2 = HealthCheckNetworkBroker::new(receiver_addr, message_receiver2, consumer_sender2);
        println!("Created message_broker_2");
        println!("Attempting to run message_broker_2");
        message_broker_2.run();
        println!("message_broker_2 finished running");
    });

    println!("Performing message send test");
    let message = HealthCheckNetworkBrokerMessage {
        payload: HealthCheckPacket {
            header: 1,
            nonce: [2,3,6,1,7,3,1,3,8,9,3,2,6,3,7,3]
        },
        remote_addr: SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT)
    };
    println!("Sending message");
    test_message_sender.send(message).expect("Message sent");
    println!("Message Sent");
    let message_from_broker = consumer_receiver2.recv().expect("Message to be received");
    println!("Received message from broker: {:?}", message_from_broker);
    println!("Waiting for threads to complete");
    message_sender_1_handle.join().expect("message_sender_1_handle joined");
    message_sender_2_handle.join().expect("message_sender_2_handle joined");
    println!("Application done running, exiting");
}

fn main_receiver_poc() {

    let receiver_handle = thread::spawn(|| {
        health_check_receiver();
    });

    let send_handle = thread::spawn(|| {
        health_check_sender();
    });

    send_handle.join().unwrap();
    receiver_handle.join().unwrap();
}

fn main_serialization() {
    let packet = HealthCheckPacket {
        header: 1,
        nonce: [2,3,6,1,7,3,1,3,8,9,3,2,6,3,7,3]
    };

    println!("Printing raw packet: ");
    println!("{:?}", packet);
    let serialized = packet.serialize();
    println!("Printing serialized packet: ");
    println!("{:?}", serialized);

    let deserialized = HealthCheckPacket::deserialize(serialized);

    println!("Printing deserialized packet: ");
    println!("{:?}", deserialized);

    println!("Deserialized packet equals original packet {}", deserialized == packet)
}
