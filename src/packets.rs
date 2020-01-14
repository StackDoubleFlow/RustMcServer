use crate::network::Client;
use std::convert::TryInto;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NetworkState {
    HANDSHAKING,
    STATUS,
    LOGIN,
    PLAY,
}

pub type PacketBuffer = Vec<u8>;
type Boolean = bool;
type Byte = i8;
type UnsignedByte = u8;
type Short = i16;
type UnsignedShort = u16;
type Int = i32;
type Long = i64;
type Float = f32;
type Double = f64;
type Chat = String; // Max length of 32767
type Indentifier = String; // Max length of 32767
type VarInt = i32;
type VarLong = i32;
type UUID = u128;
type ByteArray = Vec<u8>;

pub struct PacketDecoder {
    pub buffer: PacketBuffer,
    pub packet_id: i32,
    pub length: i32,
    i: usize,
}

impl PacketDecoder {
    pub fn new_batch(buffer: PacketBuffer, client: &Client) -> Vec<PacketDecoder> {
        let mut decoders = Vec::new();
        let mut next = buffer;
        loop {
            if client.shared_secret.is_some() {
                // TODO: Protocol Encryption
            }
            let mut decoder = PacketDecoder {
                buffer: next,
                i: 0,
                length: 0,
                packet_id: 0,
            };
            decoder.length = decoder.read_varint();
            let length_of_length = decoder.i;

            decoder.packet_id = decoder.read_varint();
            let packet_id_length = decoder.i - length_of_length;

            if (decoder.buffer.len() - length_of_length) > decoder.length as usize {
                let buffer_clone = decoder.buffer.clone();
                let (new_buffer, other_packets) =
                    buffer_clone.split_at(decoder.length as usize + decoder.i - packet_id_length);
                decoder.buffer = Vec::from(new_buffer);
                decoders.push(decoder);
                next = other_packets.to_vec();
            } else {
                decoders.push(decoder);
                break;
            }
        }
        decoders
    }

    pub fn new(buffer: PacketBuffer, client: &Client) -> PacketDecoder {
        let mut decoder = PacketDecoder {
            buffer,
            i: 0,
            length: 0,
            packet_id: 0,
        };

        decoder.length = decoder.read_varint();

        // TODO: compression
        decoder.packet_id = decoder.read_varint();

        if client.shared_secret.is_some() {
            // TODO: Protocol Encryption
        }

        decoder
    }

    fn read_ubyte(&mut self) -> u8 {
        self.i += 1;
        self.buffer[self.i - 1]
    }

    fn read_byte(&mut self) -> i8 {
        self.i += 1;
        self.buffer[self.i - 1] as i8
    }

    fn read_bytes(&mut self, bytes: usize) -> Vec<u8> {
        let out = &self.buffer[self.i..self.i + bytes];
        self.i += bytes;
        out.to_vec()
    }

    fn read_long(&mut self) -> i64 {
        let mut arr = [0; 8];
        arr.copy_from_slice(&self.buffer[self.i..self.i + 8]);
        let out = i64::from_be_bytes(arr);
        self.i += 8;
        out
    }

    fn read_int(&mut self) -> i32 {
        let mut arr = [0; 4];
        arr.copy_from_slice(&self.buffer[self.i..self.i + 4]);
        let out = i32::from_be_bytes(arr);
        self.i += 4;
        out
    }

    fn read_bool(&mut self) -> bool {
        let out;
        if self.buffer[self.i] == 1 {
            out = true;
        } else {
            out = false;
        }
        self.i += 1;
        out
    }

    fn read_varint(&mut self) -> i32 {
        let mut num_read = 0;
        let mut result = 0i32;
        let mut read;
        loop {
            read = self.read_byte() as u8;
            let value = (read & 0b01111111) as i32;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                panic!("VarInt is too big!");
            }
            if read & 0b10000000 == 0 {
                break;
            }
        }
        return result;
    }

    fn read_varlong(&mut self) -> i64 {
        let mut num_read = 0;
        let mut result = 0i64;
        let mut read;
        loop {
            read = self.read_byte() as u8;
            let value = (read & 0b01111111) as i64;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                panic!("VarInt is too big!");
            }
            if read & 0b10000000 == 0 {
                break;
            }
        }
        return result;
    }

    fn read_string(&mut self) -> String {
        let length = self.read_varint();
        String::from_utf8(self.read_bytes(length as usize)).unwrap()
    }

    fn read_ushort(&mut self) -> u16 {
        let mut arr = [0; 2];
        arr.copy_from_slice(&self.buffer[self.i..self.i + 2]);
        let out = u16::from_be_bytes(arr);
        self.i += 2;
        out
    }
}

pub struct PacketEncoder {
    buffer: PacketBuffer,
    packet_id: u8,
}

impl PacketEncoder {
    fn new(packet_id: u8) -> PacketEncoder {
        PacketEncoder {
            buffer: PacketBuffer::new(),
            packet_id,
        }
    }

    pub fn finalize(&self, compressed: bool, encryption_key: &Option<Vec<u8>>) -> Vec<u8> {
        let mut dummy_encoder = PacketEncoder::new(0);
        let mut out;

        if compressed {
            out = vec![];
        } else {
            let mut packet_id_encoder = PacketEncoder::new(0);
            packet_id_encoder.write_varint(self.packet_id as i32);
            dummy_encoder
                .write_varint(self.buffer.len() as i32 + packet_id_encoder.buffer.len() as i32);
            out = dummy_encoder.buffer.clone();
            out.append(&mut packet_id_encoder.buffer.clone());
            out.append(&mut self.buffer.clone());
        }

        if encryption_key.is_some() {
            out
        } else {
            out
        }
    }

    fn write_ubyte(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    fn write_byte(&mut self, byte: i8) {
        self.buffer.push(byte as u8);
    }
    fn write_bytes(&mut self, bytes: &mut Vec<u8>) {
        self.buffer.append(bytes);
    }

    fn write_long(&mut self, long: i64) {
        self.write_bytes(&mut long.to_be_bytes().to_vec());
    }

    fn write_int(&mut self, int: i32) {
        self.write_bytes(&mut int.to_be_bytes().to_vec());
    }
    fn write_bool(&mut self, b: bool) {
        if b {
            self.write_byte(1);
        } else {
            self.write_byte(0);
        }
    }

    fn write_short(&mut self, short: i16) {
        self.write_bytes(&mut short.to_be_bytes().to_vec());
    }

    fn write_ushort(&mut self, ushort: u16) {
        self.write_bytes(&mut ushort.to_be_bytes().to_vec());
    }

    fn write_varint(&mut self, mut value: i32) {
        loop {
            let mut temp = (value & 0b11111111) as u8;
            value = value >> 7;
            if value != 0 {
                temp |= 0b10000000;
            }
            self.write_ubyte(temp);
            if value == 0 {
                break;
            }
        }
    }

    fn write_varlong(&mut self, mut value: i64) {
        loop {
            let mut temp = (value & 0b11111111) as u8;
            value = value >> 7;
            if value != 0 {
                temp |= 0b10000000;
            }
            self.write_ubyte(temp);
            if value == 0 {
                break;
            }
        }
    }

    fn write_string(&mut self, string: String) {
        self.write_varint(string.len().try_into().unwrap());
        self.write_bytes(&mut string.as_bytes().to_vec());
    }
}

// CLIENT BOUND

pub struct C00Response {
    pub json_response: String,
}

impl C00Response {
    pub fn encode(self) -> PacketEncoder {
        let mut encoder = PacketEncoder::new(0x00);
        encoder.write_string(self.json_response);
        encoder
    }
}

pub struct C01Pong {
    pub payload: Long,
}

impl C01Pong {
    pub fn encode(self) -> PacketEncoder {
        let mut encoder = PacketEncoder::new(0x01);
        encoder.write_long(self.payload);
        encoder
    }
}

pub struct C00Disconnect {
    pub reason: Chat,
}

impl C00Disconnect {
    pub fn encode(self) -> PacketEncoder {
        let mut encoder = PacketEncoder::new(0x00);
        encoder.write_string(self.reason);
        encoder
    }
}

pub struct C01EcryptionRequest {
    pub server_id: String,
    pub public_key_length: VarInt,
    pub public_key: ByteArray,
    pub verify_token_length: VarInt,
    pub verify_token: ByteArray,
}

impl C01EcryptionRequest {
    pub fn encode(mut self) -> PacketEncoder {
        let mut encoder = PacketEncoder::new(0x01);
        encoder.write_string(self.server_id);
        encoder.write_varint(self.public_key_length);
        encoder.write_bytes(&mut self.public_key);
        encoder.write_varint(self.verify_token_length);
        encoder.write_bytes(&mut self.verify_token);
        encoder
    }
}

// SERVER BOUND

pub struct S01Ping {
    pub payload: Long,
}

impl S01Ping {
    pub fn decode(mut decoder: PacketDecoder) -> S01Ping {
        S01Ping {
            payload: decoder.read_long(),
        }
    }
}

pub struct S00Handshake {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: UnsignedShort,
    pub next_state: NetworkState,
}

impl S00Handshake {
    pub fn decode(mut decoder: PacketDecoder) -> S00Handshake {
        S00Handshake {
            protocol_version: decoder.read_varint(),
            server_address: decoder.read_string(),
            server_port: decoder.read_ushort(),
            next_state: {
                let next_state = decoder.read_varint();

                match next_state {
                    1 => NetworkState::STATUS,
                    2 => NetworkState::LOGIN,
                    _ => {
                        println!("Invalid next network state: {}", next_state);
                        NetworkState::HANDSHAKING
                    }
                }
            },
        }
    }
}

pub struct S00LoginStart {
    pub name: String,
}

impl S00LoginStart {
    pub fn decode(mut decoder: PacketDecoder) -> S00LoginStart {
        S00LoginStart {
            name: decoder.read_string(),
        }
    }
}

pub struct S01EncryptionResponse {
    pub shared_secret_length: VarInt,
    pub shared_secret: ByteArray,
    pub verify_token_length: VarInt,
    pub verify_token: ByteArray,
}

impl S01EncryptionResponse {
    pub fn decode(mut decoder: PacketDecoder) -> S01EncryptionResponse {
        let shared_secret_length = decoder.read_varint();
        let shared_secret = decoder.read_bytes(shared_secret_length.clone() as usize);
        let verify_token_length = decoder.read_varint();
        let verify_token = decoder.read_bytes(verify_token_length.clone() as usize);
        S01EncryptionResponse {
            shared_secret_length,
            shared_secret,
            verify_token_length,
            verify_token,
        }
    }
}
