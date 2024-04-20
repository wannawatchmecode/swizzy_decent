use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Mutex;
use crate::health_check::{DeserializePacket, get_health_check_opcodes, HEALTH_CHECK_ACK_OPCODE, HEALTH_CHECK_PACKET_SIZE, HEALTH_CHECK_SYN_OPCODE, HealthCheckPacket, SerializePacket};
use crate::network_models::NetworkDetails;

pub const IP: Ipv4Addr = Ipv4Addr::new(127,0,0,1);
pub const RECEIVER_PORT: u16 = 3451;
pub const SENDER_PORT: u16 = 3450;


#[derive(Debug)]
pub struct NetworkDetailsStore {
    // add id?

    // What should the key be? IP is probably best for now, can create "secondary indexes" if necessary
    host_map: Mutex<HashMap<IpAddr, NetworkDetails>>,
}

impl NetworkDetailsStore {

    pub fn new() -> NetworkDetailsStore {
        let actual_map = HashMap::new();
        return NetworkDetailsStore {
            host_map: Mutex::new(actual_map),
        }
    }

    pub fn get_network_details_by_ip(&self, ip: &IpAddr) ->  Result<NetworkDetails, ()>  {
        let host_map = self.host_map.lock().unwrap();
        let record = host_map.get(ip);
        if record.is_none() {
            return Err(())
        }

        return Ok(record.unwrap().clone())
    }

    pub fn put_network_details(&self, network_details: &NetworkDetails) {
        let mut host_map = self.host_map.lock().unwrap();
        host_map.insert(network_details.clone().addr, network_details.clone());
    }
}

// Let's write some tests
#[cfg(test)]
mod network_tests {
    use std::net::IpAddr;
    use crate::health_check_model::{HealthCheck, HealthCheckConfiguration, HealthCheckKey, HealthChecks, HealthStatus, HealthStatusDetails};
    use crate::network::{HealthCheck, HealthCheckConfiguration, HealthCheckKey, HealthChecks, HealthStatus, HealthStatusDetails, IP, NetworkDetails, NetworkDetailsStore};

    #[test]
    fn network_details_store_initializes_successfully() {
        let store = NetworkDetailsStore::new();
    }

    #[test]
    fn network_details_store_returns_data() {
        let mut store = NetworkDetailsStore::new();
        let mut health_checks = HealthChecks::new();
        health_checks.put_health_check(HealthCheckKey {port: 3000},HealthCheck {
            status_details: HealthStatusDetails {
                current_status: HealthStatus::Healthy,
                lives_remaining: 0,
            },
            configuration: HealthCheckConfiguration {
                health_check_port: 3000,
            }
        });

        let dummy_record = NetworkDetails {
            addr: IpAddr::V4(IP),
            health_checks
        };
        store.host_map.get_mut().unwrap().insert(IpAddr::V4(IP), dummy_record.clone());

        let result = store.get_network_details_by_ip(&IpAddr::V4(IP)).unwrap().clone();
        println!("{:?}", result);
        println!("{:?}", store);
        assert_eq!(dummy_record, result);
    }

    #[test]
    fn network_details_store_stores_and_returns_data() {
        let mut health_checks = HealthChecks::new();
        health_checks.put_health_check(HealthCheckKey {port: 3000},HealthCheck {
            status_details: HealthStatusDetails {
                current_status: HealthStatus::Healthy,
                lives_remaining: 0,
            },
            configuration: HealthCheckConfiguration {
                health_check_port: 3000,
            }
        });
        let dummy_record = NetworkDetails {
            addr: IpAddr::V4(IP),
            health_checks
        };
        let mut store = NetworkDetailsStore::new();
        store.put_network_details(&dummy_record);
        store.get_network_details_by_ip(&dummy_record.addr).unwrap();
        // store.host_map
    }
}

pub fn health_check_receiver() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:3451")?;

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; HEALTH_CHECK_PACKET_SIZE+1];
        let (_amt, src) = socket.recv_from(&mut buf)?;
        let buf = &buf[..HEALTH_CHECK_PACKET_SIZE];
        let buf_vec = buf.to_vec();
        let health_check_packet = HealthCheckPacket::deserialize(buf_vec);
        let mut response_object = health_check_packet.clone();
        println!("Received: {:?}", health_check_packet);
        if health_check_packet.header == HEALTH_CHECK_SYN_OPCODE {
            response_object.header = HEALTH_CHECK_ACK_OPCODE;
        } else if health_check_packet.header == HEALTH_CHECK_ACK_OPCODE {
            // TODO: update network table
        } else {
            if get_health_check_opcodes().contains(&health_check_packet.header) {
                println!("Valid op code [{}] provided, but not handled!", health_check_packet.header);
            } else {
                println!("Invalid op code received of [{}]", health_check_packet.header);
            }
            return Ok(());
        }
        // Redeclare `buf` as slice of the received data and send reverse data back to origin.
        // let buf = &mut buf[..amt];
        // buf.reverse();

        let buf = &mut [0;HEALTH_CHECK_PACKET_SIZE];
        let raw = response_object.serialize();
        for i in 0..HEALTH_CHECK_PACKET_SIZE {
            buf[i] = raw[i];
        }

        socket.send_to(buf, &src)?;
    } // the socket is closed here
    Ok(())
}

pub fn health_check_sender() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:3450")?;
        let request_object = HealthCheckPacket {
            header: HEALTH_CHECK_SYN_OPCODE,
            nonce: [2,3,6,1,7,3,2,3,4,9,3,2,1,7,7,3]
        };
        let buf = &mut [0;HEALTH_CHECK_PACKET_SIZE];
        let raw = request_object.serialize();
        for i in 0..HEALTH_CHECK_PACKET_SIZE {
            buf[i] = raw[i];
        }
        let dst = SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT);
        let _amt = socket.send_to(buf, &dst)?;

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.

        let mut buf = [0; HEALTH_CHECK_PACKET_SIZE+1];

        let (_amt, _src) = socket.recv_from(&mut buf)?;

        let buf = &buf[..HEALTH_CHECK_PACKET_SIZE];
        let buf_vec = buf.to_vec();
        let health_check_packet = HealthCheckPacket::deserialize(buf_vec);
        let mut response_object = health_check_packet.clone();
        println!("Received: {:?}", response_object);
        if health_check_packet.header == HEALTH_CHECK_SYN_OPCODE {
            response_object.header = HEALTH_CHECK_ACK_OPCODE;
        } else if health_check_packet.header == HEALTH_CHECK_ACK_OPCODE {
            // println!("Health check ack op code received from {}")
            // TODO: update network table
        } else {
            if get_health_check_opcodes().contains(&health_check_packet.header) {
                println!("Valid op code [{}] provided, but not handled!", health_check_packet.header);
            } else {
                println!("Invalid op code received of [{}]", health_check_packet.header);
            }
            return Ok(());
        }
    } // the socket is closed here
    Ok(())
}