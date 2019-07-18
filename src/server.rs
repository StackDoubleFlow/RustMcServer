
use packets::PacketFactory;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::collections::HashMap;

struct NetworkClient {
  tcp_stream: TcpStream,
  addr: SocketAddr
}

impl NetworkClient {
  fn new(stream: TcpStream) -> NetworkClient {
    NetworkClient {
      addr: stream.peer_addr().unwrap(),
      tcp_stream: stream
    }
  }

  pub fn listen_for_packets(&self) {

  }
}

struct MinecraftServer {
  is_running: bool,
  clients: HashMap<SocketAddr, NetworkClient>,
  listener: TcpListener,
}

impl MinecraftServer {
  fn start_server(port: u16) {
    println!("Starting server!");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let server = MinecraftServer {
      clients: HashMap::new(),
      is_running: true,
      listener: TcpListener::bind(addr).unwrap(),
    };
    println!("Server started on port {}", port);
    server.listen_for_connections();
  }

  fn handle_connection(&mut self, stream: TcpStream) {
    let nc = NetworkClient::new(stream);
    self.clients.insert(nc.addr, nc);
    thread::spawn(|| {
      nc.listen_for_packets();
    });
  }

  fn listen_for_connections(&self) {
    for stream in self.listener.incoming() {
      self.handle_connection(stream.unwrap());
    }
  }
}


fn handle_connection(mut stream: TcpStream) {
  
}


pub fn run_server() {

  MinecraftServer::start_server(25565);

}