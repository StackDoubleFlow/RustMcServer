mod mojang;
mod network;
mod packets;
mod player;
mod utils;
mod world;

fn main() {
    println!("Loading world...");
    network::start_server();
}
