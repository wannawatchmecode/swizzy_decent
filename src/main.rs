use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::mpsc;
use std::{env, io, thread};
use std::io::BufRead;
use std::str::FromStr;
use std::thread::{sleep, Thread};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::Local;
use env_logger::Builder;
use std::io::Write;
use log::{info, LevelFilter};
use clap::Parser;
use crate::cli::{SwizzyDecentCli, SwizzyDecentCliRunConfiguration};
use crate::health_check::{DeserializePacket, HEALTH_CHECK_SYN_OPCODE, HealthCheckPacket, SerializePacket};
use crate::health_check_network_broker::{build_health_check_stack, HealthCheckNetworkBroker, HealthCheckNetworkBrokerMessage};
use crate::health_check_network_handlers::HealthCheckNetworkBrokerMessageListener;
use crate::network::{health_check_receiver, health_check_sender, IP, NetworkDetailsStore, RECEIVER_PORT, SENDER_PORT};
use crate::utils::generate_nonce;

mod health_check;
mod network;
mod health_check_network_broker;
mod health_check_network_handlers;
mod example;
mod utils;
mod cli;
mod network_models;
mod health_check_model;
mod print_network_command;

const IP_ADDRESS_ENV_KEY: &str = "HEALTH_CHECK_IP_ADDRESS";
const UDP_PORT_ENV_KEY: &str = "HEALTH_CHECK_UDP_PORT";
const UDP_PORT_DEFAULT: u16 = 3450;
const DEFAULT_IP_ADDRESS: &str = "127.0.0.1";

fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                     "{} [{}] - {}",
                     Local::now().format("%Y-%m-%dT%H:%M:%S"),
                     record.level(),
                     record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
    // main_with_stacks()
    // main_health_check_broker_example()
    // main_load_example()
    // main_with_receiver_handler()
    // main_receiver_poc()
    single_instance_main()
}

#[derive(Debug, Parser)]
struct HealthCheckCli {
    /// Input file to read
    socket_addr: SocketAddr,
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
    let network_broker = &stack.network_broker;
    let request_sender = stack.request_sender.clone();

    let stack_handle = thread::spawn(move || {
        stack.run();
    });

    info!("Started health check server on {}:{}", ip_address_str, listener_port);

    // let cli_handle = thread::spawn(move || {
    //     let stdin = io::stdin();
    //
    //     loop {
    //         // let args = HealthCheckCli::parse();
    //         for line in stdin.lock().lines() {
    //             println!("{:?}", line);
    //             // parse_command(line);
    //             let start = SystemTime::now();
    //             let since_the_epoch = start
    //                 .duration_since(UNIX_EPOCH)
    //                 .expect("Time went backwards");
    //             request_sender.send(HealthCheckNetworkBrokerMessage {
    //                 payload: HealthCheckPacket {
    //                     header: HEALTH_CHECK_SYN_OPCODE,
    //                     nonce: generate_nonce()
    //                 },
    //                 remote_addr: sender_addr,
    //             }).unwrap();
    //
    //             let end = SystemTime::now();
    //             let end_since_the_epoch = end
    //                 .duration_since(UNIX_EPOCH)
    //                 .expect("Time went backwards");
    //             println!("Start: [{:?}] End: [{:?}] TotalDuration: [{:?}]", since_the_epoch, end_since_the_epoch, end_since_the_epoch-since_the_epoch);
    //         }
    //     }
    // });
    //
    // cli_handle.join().unwrap();

    // stack.network_broker
    // let network_broker ;
    let cli = SwizzyDecentCli::new(request_sender);
    let cli_handle = thread::spawn(move || {
        cli.run(SwizzyDecentCliRunConfiguration {});
    });

    cli_handle.join().unwrap();
    stack_handle.join().unwrap();
}

fn parse_command(line: &str) {

}

