
use crate::player::Player;
use crate::utils::Location;

pub struct World {}

impl World {

    pub fn load_world() -> World {
        World {}
    }

    pub fn load_player(&self, username: String) -> Player {
        Player {
            username,
            position: Location::new(0, 0, 0, 0, 0)
        }
    }
}