use std::convert::TryInto;

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
    buffer: PacketBuffer,
    i: usize,
}

impl PacketDecoder {
    pub fn new(buffer: PacketBuffer) -> PacketDecoder {
        PacketDecoder { buffer, i: 0 }
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
        let mut numRead = 0;
        let mut result = 0i32;
        let mut read;
        loop {
            read = self.read_byte() as u8;
            let value = (read & 0b01111111) as i32;
            result |= value << (7 * numRead);

            numRead += 1;
            if numRead > 5 {
                panic!("VarInt is too big!");
            }
            if read & 0b10000000 == 0 {
                break;
            }
        }
        return result;
    }

    fn read_varlong(&mut self) -> i64 {
        let mut numRead = 0;
        let mut result = 0i64;
        let mut read;
        loop {
            read = self.read_byte() as u8;
            let value = (read & 0b01111111) as i64;
            result |= value << (7 * numRead);

            numRead += 1;
            if numRead > 5 {
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

struct PacketEncoder {
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

    fn finalize(&self, _compressed: bool, _encryption_key: Option<Vec<u8>>) -> Vec<u8> {
        self.buffer.clone()
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
    json_response: String,
}

impl C00Response {
    pub fn encode(self) -> PacketEncoder {
        let mut encoder = PacketEncoder::new(0x00);
        encoder.write_string(self.json_response);
        encoder
    }
}

pub struct C01Pong {
    payload: Long,
}

impl C01Pong {
    pub fn encode(self, payload: Long) -> PacketEncoder {
        let mut encoder = PacketEncoder::new(0x01);
        encoder.write_long(payload);
        encoder
    }
}

pub struct C00Disconnect {
    reason: Chat,
}

impl C00Disconnect {
    pub fn encode(self) -> PacketEncoder {
        let mut encoder = PacketEncoder::new(0x00);
        encoder.write_string(self.reason);
        encoder
    }
}

pub struct C01EcryptionRequest {
    server_id: String,
    public_key_length: VarInt,
    public_key: ByteArray,
    verify_token_length: VarInt,
    verify_token: ByteArray,
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

pub struct S00Request {}

pub struct S01Ping {
    payload: Long,
}

impl S01Ping {
    pub fn decode(decoder: &mut PacketDecoder) -> S01Ping {
        S01Ping {
            payload: decoder.read_long(),
        }
    }
}

pub struct S00Handshake {
    protocol_version: VarInt,
    server_address: String,
    server_port: UnsignedShort,
    next_state: NetworkState,
}

impl S00Handshake {
    pub fn decode(decoder: &mut PacketDecoder) -> S00Handshake {
        S00Handshake {
            protocol_version: decoder.read_varint(),
            server_address: decoder.read_string(),
            server_port: decoder.read_ushort(),
            next_state: match decoder.read_varint() {
                1 => NetworkState::STATUS,
                2 => NetworkState::LOGIN,
                _ => panic!("Invalid next network state"),
            },
        }
    }
}
