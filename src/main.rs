mod network;
mod packets;
mod utils;
mod world;

fn main() {
    println!("Loading world...");
    network::start_server();
}
