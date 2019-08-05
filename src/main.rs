
mod network;
mod packets;
mod utils;
mod world;

fn main() {
    println!("Starting server");
    network::start_server();
}
