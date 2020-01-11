
use crate::utils::Vec3;
use crate::utils::Location;

pub struct Entity {
    velocity: Vec3,
    location: Location,
    height: f64,
    width: f64,
    on_ground: bool,
    id: i32,
    fire_ticks: i32,
    max_fire_ticks: i32,
    persistent: bool,
    passenger: Entity,
    fall_distance: f32,
    uuid: u128,
    ticks_lived: i32,
    custom_name_visible: bool,
    name_visible: String,
    glowing: bool,
    invulnerable: bool,
    silent: bool,
    gravity: bool,
    portal_cooldown: i32,
    scoreboard_tags: Vec<String>,
}

impl Entity {
  pub fn new() -> Self {
    
  }
}