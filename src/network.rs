use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use crate::health_check::{DeserializePacket, get_health_check_opcodes, HEALTH_CHECK_ACK_OPCODE, HEALTH_CHECK_PACKET_SIZE, HEALTH_CHECK_SYN_OPCODE, HealthCheckPacket, SerializePacket};

pub const IP: Ipv4Addr = Ipv4Addr::new(127,0,0,1);
pub const RECEIVER_PORT: u16 = 3451;
pub const SENDER_PORT: u16 = 3450;
pub fn health_check_receiver() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:3451")?;

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; HEALTH_CHECK_PACKET_SIZE+1];
        let (amt, src) = socket.recv_from(&mut buf)?;
        let mut buf = &buf[..HEALTH_CHECK_PACKET_SIZE];
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

        let mut buf = &mut [0;HEALTH_CHECK_PACKET_SIZE];
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
        let mut buf = &mut [0;HEALTH_CHECK_PACKET_SIZE];
        let raw = request_object.serialize();
        for i in 0..HEALTH_CHECK_PACKET_SIZE {
            buf[i] = raw[i];
        }
        let dst = SocketAddr::new(IpAddr::V4(IP), RECEIVER_PORT);
        let amt = socket.send_to(buf, &dst)?;

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.

        let mut buf = [0; HEALTH_CHECK_PACKET_SIZE+1];

        let (amt, src) = socket.recv_from(&mut buf)?;

        let mut buf = &buf[..HEALTH_CHECK_PACKET_SIZE];
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
