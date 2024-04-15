use std::{env, io, thread};
use std::io::BufRead;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::{DEFAULT_IP_ADDRESS, IP_ADDRESS_ENV_KEY, UDP_PORT_DEFAULT, UDP_PORT_ENV_KEY};
use crate::health_check::{DeserializePacket, HEALTH_CHECK_SYN_OPCODE, HealthCheckPacket, SerializePacket};
use crate::health_check_network_broker::{build_health_check_stack, HealthCheckNetworkBroker, HealthCheckNetworkBrokerMessage};
use crate::health_check_network_handlers::HealthCheckNetworkBrokerMessageListener;
use crate::network::{health_check_receiver, health_check_sender, IP, RECEIVER_PORT, SENDER_PORT};


fn main_with_stacks() {
    let sender_addr = SocketAddr::new(IpAddr::V4(IP), SENDER_PORT);
    let stack = build_health_check_stack(sender_addr);
    // let network_broker1 = &stack.network_broker;

    let first_handle = thread::spawn(|| {
        stack.run();
    });

    let sender_addr2 = SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT);
    let stack2 = build_health_check_stack(sender_addr2);
    let network_broker2 = &stack2.network_broker;
    let test_message_sender = network_broker2.request_sender.clone();
    // let network_broker2 = network_broker2;
    let second_handle = thread::spawn( || {
        stack2.run();
        // network_broker2.run();
    });


    let message = HealthCheckNetworkBrokerMessage {
        payload: HealthCheckPacket {
            header: 1,
            nonce: [2,3,6,1,7,3,1,3,8,9,3,2,6,3,7,3]
        },
        remote_addr: SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT)
    };

    // println!("Sending message");
    // test_message_sender.send(message).expect("Message sent");
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let CALLS = 10;
    for _i in 0..CALLS {
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


    thread::sleep(Duration::from_secs(3));
    println!("Start: [{:?}] End: [{:?}] TotalDuration: [{:?}]", since_the_epoch, end_since_the_epoch, end_since_the_epoch-since_the_epoch);

    first_handle.join();
    second_handle.join();
}

/**
Starts a single health check server instance.
Pulls configuration from env variables, with defaults.

Defaults:

Default port = 3450
Default IP = 127.0.0.1
 */
fn single_instance_main() {

    let ip_address_str = env::var(IP_ADDRESS_ENV_KEY).unwrap_or(String::from(DEFAULT_IP_ADDRESS));
    let listener_port_str = env::var(UDP_PORT_ENV_KEY).unwrap_or(UDP_PORT_DEFAULT.to_string());
    let listener_port: u16 = listener_port_str.parse().expect("Valid string number");
    let ip = IpAddr::from_str(&ip_address_str.as_str()).expect("Valid IP address");

    let sender_addr = SocketAddr::new(ip, listener_port);
    let stack = build_health_check_stack(sender_addr);
    // let network_broker = stack.network_broker;
    // let request_sender = network_broker.get_request_sender().clone();
    let request_sender = stack.request_sender.clone();
    let stack_handle = thread::spawn(move || {
        stack.run();
    });

    println!("Started health check server on {}:{}", ip_address_str, listener_port);

    let cli_handle = thread::spawn(move || {
        let stdin = io::stdin();
        loop {
            for line in stdin.lock().lines() {
                println!("{}", line.unwrap());
                request_sender.send(HealthCheckNetworkBrokerMessage {
                    payload: HealthCheckPacket {
                        header: HEALTH_CHECK_SYN_OPCODE,
                        nonce: [2,3,6,1,7,3,1,3,8,9,3,2,6,3,7,3]
                    },
                    remote_addr: sender_addr,

                }).unwrap();
            }
        }
    });

    cli_handle.join().unwrap();
    stack_handle.join().unwrap();
}

// fn main_load_example() {
//     println!("HealthCheckBroker with receiver handler load example main");
//
//     let message_sender_1_handle = thread::spawn(move || {
//         let (message_sender1, message_receiver1) = mpsc::channel();
//         let sender_addr = SocketAddr::new(IpAddr::V4(IP), SENDER_PORT);
//         let message_broker_1 = HealthCheckNetworkBroker::new(sender_addr,  mpsc::channel().0, message_receiver1, message_sender1);
//         println!("Created message_broker_1");
//         println!("Attempting to run message_broker_1");
//         message_broker_1.run();
//         println!("message_broker_1 finished running");
//     });
//
//     let (message_sender2, message_receiver2) = mpsc::channel();
//     let (consumer_sender2, consumer_receiver2) = mpsc::channel();
//
//     let test_message_sender = message_sender2.clone();
//     let message_sender_2_handle = thread::spawn(move || {
//         let _message_sender = message_sender2.clone();
//         let receiver_addr = SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT);
//         let message_broker_2 = HealthCheckNetworkBroker::new(receiver_addr, consumer_sender2.clone() /* Not intended to be used in this example*/, message_receiver2, consumer_sender2);
//         println!("Created message_broker_2");
//         println!("Attempting to run message_broker_2");
//         message_broker_2.run();
//         println!("message_broker_2 finished running");
//     });
//     let message_sender2 = test_message_sender.clone();
//
//     let receiver_handle = thread::spawn(move || {
//         let broker_message_listener2 = HealthCheckNetworkBrokerMessageListener::new(consumer_receiver2, message_sender2);
//         broker_message_listener2.run();
//     });
//
//     // Testing
//     println!("Performing message send test");
//
//     let start = SystemTime::now();
//     let since_the_epoch = start
//         .duration_since(UNIX_EPOCH)
//         .expect("Time went backwards");
//     println!("{:?}", since_the_epoch);
//
//     for i in 0..1000 {
//         let message = HealthCheckNetworkBrokerMessage {
//             payload: HealthCheckPacket {
//                 header: 1,
//                 nonce: [2,3,6,1,7,3,1,3,8,9,3,2,6,3,7,3]
//             },
//             remote_addr: SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT)
//         };
//
//         // println!("Sending message");
//         test_message_sender.send(message).expect("Message sent");
//         // println!("Message Sent");
//     }
//
//     let end = SystemTime::now();
//     let end_since_the_epoch = end
//         .duration_since(UNIX_EPOCH)
//         .expect("Time went backwards");
//     println!("Start: [{:?}] End: [{:?}] TotalDuration: [{:?}]", since_the_epoch, end_since_the_epoch, end_since_the_epoch-since_the_epoch);
//     message_sender_2_handle.join().expect("message_sender_2_handle joined");
//
//     // let message_from_broker = consumer_receiver2.recv().expect("Message to be received");
//     // println!("Received message from broker: {:?}", message_from_broker);
//     println!("Waiting for threads to complete");
//     receiver_handle.join().expect("Receiver_handle joined");
//     message_sender_1_handle.join().expect("message_sender_1_handle joined");
//     // message_sender_2_handle.join().expect("message_sender_2_handle joined");
//     println!("Application done running, exiting");
// }

// fn main_with_receiver_handler() {
//     println!("HealthCheckBroker with receiver handler example main");
//
//     let message_sender_1_handle = thread::spawn(move || {
//         let (message_sender1, message_receiver1) = mpsc::channel();
//         let sender_addr = SocketAddr::new(IpAddr::V4(IP), SENDER_PORT);
//         let message_broker_1 = HealthCheckNetworkBroker::new(sender_addr, message_sender1.clone() /* Not intended to be used in this example*/, message_receiver1, message_sender1);
//         println!("Created message_broker_1");
//         println!("Attempting to run message_broker_1");
//         message_broker_1.run();
//         println!("message_broker_1 finished running");
//     });
//
//     let (message_sender2, message_receiver2) = mpsc::channel();
//     let (consumer_sender2, consumer_receiver2) = mpsc::channel();
//
//     let test_message_sender = message_sender2.clone();
//     let message_sender_2_handle = thread::spawn(move || {
//         let _message_sender = message_sender2.clone();
//         let receiver_addr = SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT);
//         let message_broker_2 = HealthCheckNetworkBroker::new(receiver_addr, message_sender2.clone() /* Not intended to be used in this example*/, message_receiver2, consumer_sender2);
//         println!("Created message_broker_2");
//         println!("Attempting to run message_broker_2");
//         message_broker_2.run();
//         println!("message_broker_2 finished running");
//     });
//     let message_sender2 = test_message_sender.clone();
//
//     let receiver_handle = thread::spawn(move || {
//         let broker_message_listener2 = HealthCheckNetworkBrokerMessageListener::new(consumer_receiver2, message_sender2);
//         broker_message_listener2.run();
//     });
//
//     // Testing
//     println!("Performing message send test");
//     let message = HealthCheckNetworkBrokerMessage {
//         payload: HealthCheckPacket {
//             header: 1,
//             nonce: [2,3,6,1,7,3,1,3,8,9,3,2,6,3,7,3]
//         },
//         remote_addr: SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT)
//     };
//     println!("Sending message");
//     test_message_sender.send(message).expect("Message sent");
//     println!("Message Sent");
//     // let message_from_broker = consumer_receiver2.recv().expect("Message to be received");
//     // println!("Received message from broker: {:?}", message_from_broker);
//     println!("Waiting for threads to complete");
//     receiver_handle.join().expect("Receiver_handle joined");
//     message_sender_1_handle.join().expect("message_sender_1_handle joined");
//     message_sender_2_handle.join().expect("message_sender_2_handle joined");
//     println!("Application done running, exiting");
// }

fn main_health_check_broker_example() {
    println!("HealthCheckBroker example main");

    let message_sender_1_handle = thread::spawn(move || {
        let (message_sender1, message_receiver1) = mpsc::channel();
        let sender_addr = SocketAddr::new(IpAddr::V4(IP), SENDER_PORT);
        let message_broker_1 = HealthCheckNetworkBroker::new(sender_addr, message_sender1.clone() /* Not intended to be used in this example*/, message_receiver1, message_sender1);
        println!("Created message_broker_1");
        println!("Attempting to run message_broker_1");
        message_broker_1.run();
        println!("message_broker_1 finished running");
    });

    let (message_sender2, message_receiver2) = mpsc::channel();
    let (consumer_sender2, consumer_receiver2) = mpsc::channel();

    let test_message_sender = message_sender2.clone();
    let message_sender_2_handle = thread::spawn(move || {
        let _message_sender = message_sender2.clone();
        let receiver_addr = SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT);
        let message_broker_2 = HealthCheckNetworkBroker::new(receiver_addr, message_sender2.clone() /* Not intended to be used in this example*/, message_receiver2, consumer_sender2);
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
