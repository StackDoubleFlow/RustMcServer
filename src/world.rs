
use crate::player::Player;

pub struct World {}

impl World {

    pub fn load_world() -> World {
        World {}
    }

    pub fn load_player(&self, username: String) -> Player {
        Player {
            username
        }
    }
}