use crate::world::World;
use openssl::sha::Sha1;
use serde_json::Map;

pub struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3 {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

pub struct Location {
    x: i32,
    y: i32,
    z: i32,
    pitch: i8,
    yaw: i8,
}

impl Location {
    pub fn new(x: i32, y: i32, z: i32, pitch: i8, yaw: i8) -> Self {
        Self { x, y, z, pitch, yaw }
    }
}

pub struct ChatComponent {
    text: String,
    children: Vec<String>,
    color: String,
    bold: bool,
    underline: bool,
    italics: bool,
    obfuscated: bool,
}

impl ChatComponent {
    fn new(text: String) -> Self {
        Self { 
            text, 
            children: Vec::new(), 
            color: "none".to_string(), 
            bold: false, 
            underline: false, 
            italics: false, 
            obfuscated: false 
        }
    }

    pub fn build() -> Option<Map<String, bool>> {
        return None;
    }
}

pub fn to_hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    strs.join("")
}

fn mc_twos_comp(bytes: &mut Vec<u8>) {
    let mut carry = true;
    for i in (0..bytes.len()).rev() {
        bytes[i] = !bytes[i] & 0xff;
        if carry {
            carry = bytes[i] == 0xff;
            bytes[i] += 1;
        }
    }
}

pub fn mc_hex_digest(name: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(name.as_bytes());
    let mut bytes = hasher.finish().to_vec();
    let negative = (bytes[0] & 0x80) == 0x80;
    if negative {
        mc_twos_comp(&mut bytes);
        format!("-{}", String::from(to_hex_string(bytes).trim_start_matches("0")))
    } else {
        String::from(to_hex_string(bytes).trim_start_matches("0"))
    }
}