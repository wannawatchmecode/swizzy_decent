use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::mpsc;
use std::{env, io, thread};
use std::io::BufRead;
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::health_check::{DeserializePacket, HEALTH_CHECK_SYN_OPCODE, HealthCheckPacket, SerializePacket};
use crate::health_check_network_broker::{build_health_check_stack, HealthCheckNetworkBroker, HealthCheckNetworkBrokerMessage};
use crate::health_check_network_handlers::HealthCheckNetworkBrokerMessageListener;
use crate::network::{health_check_receiver, health_check_sender, IP, NetworkDetailsStore, RECEIVER_PORT, SENDER_PORT};

mod health_check;
mod network;
mod health_check_network_broker;
mod health_check_network_handlers;
mod example;

const IP_ADDRESS_ENV_KEY: &str = "HEALTH_CHECK_IP_ADDRESS";
const UDP_PORT_ENV_KEY: &str = "HEALTH_CHECK_UDP_PORT";
const UDP_PORT_DEFAULT: u16 = 3450;
const DEFAULT_IP_ADDRESS: &str = "127.0.0.1";

fn main() {
    // main_with_stacks()
    // main_health_check_broker_example()
    // main_load_example()
    // main_with_receiver_handler()
    // main_receiver_poc()
    single_instance_main()
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