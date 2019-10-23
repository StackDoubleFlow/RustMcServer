mod network;
mod packets;
mod player;
mod utils;
mod world;
mod player;

fn main() {
    println!("Loading world...");
    network::start_server();
}
