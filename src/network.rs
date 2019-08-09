extern crate openssl;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use openssl::rsa::{Padding, Rsa};

use crate::packets::*;
use crate::world::World;

struct Connection {
    packets: Arc<Mutex<Vec<PacketBuffer>>>,
    stream: TcpStream,
}

impl Connection {
    fn new(mut stream: TcpStream) -> Connection {
        println!("New connection!");
        let mut reader = stream.try_clone().unwrap();
        let connection = Connection {
            packets: Arc::new(Mutex::new(Vec::new())),
            stream,
        };

        let packets_clone = connection.packets.clone();

        thread::spawn(|| {
            Connection::handle_connection(reader, packets_clone);
        });
        connection
    }

    fn handle_connection(mut reader: TcpStream, packets: Arc<Mutex<Vec<PacketBuffer>>>) {
        loop {
            let mut data = vec![0u8; 512];
            let length = reader.read(&mut data).unwrap();
            if length == 0 {
                return;
            }
            data.drain(0..length);
            data.shrink_to_fit();
            packets.lock().unwrap().push(data);
        }
    }
}

struct Client {
    connection: Connection,
    state: NetworkState,
}

impl Client {
    fn new(stream: TcpStream) -> Client {
        let connection = Connection::new(stream);
        Client {
            connection,
            state: NetworkState::HANDSHAKING,
        }
    }
}

struct Server {
    clients: Arc<Mutex<Vec<Client>>>,
}

impl Server {
    fn new() -> Server {
        Server {
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }
    fn listen_for_connections(&self) {
        let clients = self.clients.clone();
        thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:25566").unwrap();
            for stream in listener.incoming() {
                let stream = stream.unwrap();

                let mut client = Client::new(stream);
                clients.lock().unwrap().push(client);
            }
        });
    }

    fn handle_packet(
        &mut self,
        client: &Client,
        clients: &MutexGuard<'_, std::vec::Vec<Client>>,
        packet: PacketBuffer,
    ) {
        let decoder = PacketDecoder::new(packet);
        println!("Packet received: {}", decoder.packet_id);
        match client.state {
            NetworkState::HANDSHAKING => match decoder.packet_id {
                0x00 => {
                    let packet = S00Handshake::decode(decoder);
                    client.state = packet.next_state;
                }
                _ => {
                    println!("Unknown packet with id: {}", decoder.packet_id);
                }
            },
            NetworkState::STATUS => {}
            NetworkState::LOGIN => {}
            NetworkState::PLAY => {}
        }
    }

    fn check_incoming_packets(&mut self) {
        let clients = self.clients.lock().unwrap();
        for client in clients.iter() {
            let mut packets = client.connection.packets.lock().unwrap();
            for packet in packets.drain(..) {
                self.handle_packet(client, &clients, packet);
            }
        }
    }

    fn start(mut self) {
        let rsa = Rsa::generate(1024).unwrap();
        self.listen_for_connections();
        loop {
            self.check_incoming_packets();
        }
    }
}

pub fn start_server() {
    println!("Starting server...");
    let mut server = Server::new();
    server.start();
}
