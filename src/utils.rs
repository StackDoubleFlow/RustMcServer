use crate::world::World;
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
    world: World,
}

impl Location {
    fn new(x: i32, y: i32, z: i32, pitch: i8, yaw: i8, world: World) -> Self {
        Self { x, y, z, pitch, yaw, world }
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
