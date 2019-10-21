mod network;
mod packets;
mod utils;
mod world;
mod player;

fn main() {
    println!("Loading world...");
    network::start_server();
}
