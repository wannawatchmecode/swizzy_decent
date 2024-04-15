use std::collections::HashSet;

const HEADER_SIZE_BYTES: usize = 1;
const NONCE_SIZE_BYTES: usize = 16;
pub const HEALTH_CHECK_PACKET_SIZE: usize = HEADER_SIZE_BYTES + NONCE_SIZE_BYTES;
const HEADER_INDEX: usize = 0;
const NONCE_INDEX: usize = 1;

pub const NOOP_OPCODE: u8 = 0;

pub const HEALTH_CHECK_SYN_OPCODE: u8 = 1;
pub const HEALTH_CHECK_ACK_OPCODE: u8 = 2;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HealthCheckPacket {
    pub header: u8,
    pub nonce: [u8; NONCE_SIZE_BYTES]
}

pub trait SerializePacket {
    fn serialize(&self) -> Vec<u8>;
}

pub fn get_health_check_opcodes() -> HashSet<u8> {
    return HashSet::from([HEALTH_CHECK_SYN_OPCODE, HEALTH_CHECK_ACK_OPCODE]);
}

pub trait DeserializePacket {
    // const HEALTH_CHECK_OPCODES: HashSet<u8> =

    fn deserialize(raw: Vec<u8>) -> HealthCheckPacket;
}


impl SerializePacket for HealthCheckPacket {
    fn serialize(&self) -> Vec<u8>
    {
        let header: u8 = self.header;
        let nonce: Vec<u8> = Vec::from(self.nonce);
        let mut serialized: Vec<u8> = Vec::from([header]);
        serialized.extend(nonce);
        return serialized
    }
}

impl DeserializePacket for HealthCheckPacket {
    fn deserialize(raw: Vec<u8>) -> HealthCheckPacket
    {

        if raw.len() != HEALTH_CHECK_PACKET_SIZE {
            println!("Invalid raw packet attempted to deserialize for HealthCheckPacket");
            return HealthCheckPacket {
                header: NOOP_OPCODE,
                nonce: [0; 16]
            }
        }

        let header: u8 = raw[HEADER_INDEX];
        if !get_health_check_opcodes().contains(&header) {
            println!("Invalid op code received");
            return HealthCheckPacket {
                header: NOOP_OPCODE,
                nonce: [0; 16]
            }
        }

        let mut nonce = [0;NONCE_SIZE_BYTES];
        for i in 0..NONCE_SIZE_BYTES {
            nonce[i] = raw[i+NONCE_INDEX];
        }

        let result = HealthCheckPacket {
            header: header.into(),
            nonce
        };
        return result
    }
}

#[cfg(test)]
mod health_check_tests {
    use crate::health_check::{DeserializePacket, HEALTH_CHECK_SYN_OPCODE, HealthCheckPacket, NOOP_OPCODE, SerializePacket};

    #[test]
    fn serialize_happy_case() {
        let packet = HealthCheckPacket {
            header: HEALTH_CHECK_SYN_OPCODE,
            nonce: [2,3,6,1,7,3,1,3,8,9,3,2,6,3,7,3]
        };

        let serialized = packet.serialize();
        let expected: Vec<u8> = Vec::from([1, 2, 3, 6, 1, 7, 3, 1, 3, 8, 9, 3, 2, 6, 3, 7, 3]);
        assert_eq!(expected, serialized, "We expect the serialized data: {:?} to equal the expected vector: {:?}", serialized, expected);
    }

    #[test]
    fn deserialize_happy_case() {
        let serialized: Vec<u8> = Vec::from([1, 2, 3, 6, 1, 7, 3, 1, 3, 8, 9, 3, 2, 6, 3, 7, 3]);

        let expected = HealthCheckPacket {
            header: HEALTH_CHECK_SYN_OPCODE,
            nonce: [2,3,6,1,7,3,1,3,8,9,3,2,6,3,7,3]
        };

        let deserialized = HealthCheckPacket::deserialize(serialized);
        assert_eq!(expected, deserialized, "We expect the deserialized data: {:?} to equal the expected HealthCheckPacket: {:?}", deserialized, expected);
    }

    // Non happy cases
    #[test]
    fn deserialize_packet_size_too_small() {
        // expect no op opcode with 0'd data
        let serialized: Vec<u8> = Vec::from([1, 2, 3, 6, 1, 7, 3, 1, 3, 8, 9, 3, 2, 6, 3, 7]);

        let expected = HealthCheckPacket {
            header: NOOP_OPCODE,
            nonce: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
        };

        let deserialized = HealthCheckPacket::deserialize(serialized);
        assert_eq!(expected, deserialized, "We expect the deserialized data: {:?} to equal the expected HealthCheckPacket: {:?} , with NOOP code and 0'd data when data too small", deserialized, expected);
    }

    #[test]
    fn deserialize_packet_size_too_big() {
        // expect no op opcode with 0'd data
        let serialized: Vec<u8> = Vec::from([1, 2, 3, 6, 1, 7, 3, 1, 3, 8, 9, 3, 2, 6, 3, 7, 3, 9]);

        let expected = HealthCheckPacket {
            header: NOOP_OPCODE,
            nonce: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
        };

        let deserialized = HealthCheckPacket::deserialize(serialized);
        assert_eq!(expected, deserialized, "We expect the deserialized data: {:?} to equal the expected HealthCheckPacket: {:?} , with NOOP code and 0'd data when data too large", deserialized, expected);
    }

    #[test]
    fn deserialize_invalid_op_code() {
        // expect no op opcode with 0'd data
        // expect no op opcode with 0'd data
        let serialized: Vec<u8> = Vec::from([3, 2, 3, 6, 1, 7, 3, 1, 3, 8, 9, 3, 2, 6, 3, 7, 3]);

        let expected = HealthCheckPacket {
            header: NOOP_OPCODE,
            nonce: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
        };

        let deserialized = HealthCheckPacket::deserialize(serialized);
        assert_eq!(expected, deserialized, "We expect the deserialized data: {:?} to equal the expected HealthCheckPacket: {:?} , with NOOP code and 0'd data when opcode is invalid", deserialized, expected);
    }
}