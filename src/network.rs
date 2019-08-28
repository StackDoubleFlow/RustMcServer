extern crate openssl;
use crate::packets::*;
use crate::world::World;
use openssl::rsa::{Padding, Rsa};
use serde_json::json;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;


struct Connection {
    packets: Arc<Mutex<Vec<PacketBuffer>>>,
    stream: TcpStream,
}

impl Connection {
    fn new(stream: TcpStream) -> Connection {
        println!("New connection!");
        let reader = stream.try_clone().unwrap();
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
            println!("Data length: {}", length);
            data.drain(length..);
            data.shrink_to_fit();
            //for i in 0..data.len() {
            //    print!("{:x}", data[i]);
            //}
            //println!("");
            packets.lock().unwrap().push(data);
        }
    }

    fn receive_packets(&self) -> Vec<PacketBuffer> {
        let mut packets = self.packets.lock().unwrap();
        let out = packets.clone();
        packets.clear();
        out
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

    fn send_packet(&mut self, buffer: PacketBuffer) {
        self.connection.stream.write(buffer.as_slice()).unwrap();
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

                let client = Client::new(stream);
                clients.lock().unwrap().push(client);
            }
        });
    }

    fn unknown_packet(id: i32) {
        println!("Unknown packet with id: {}", id);
    }

    fn handle_packet(&mut self, client: usize, packet: PacketBuffer) {
        let mut clients = self.clients.lock().unwrap();
        let (decoder, other_packets) = PacketDecoder::new(packet);
        println!(
            "Packet received: {}, with the length of: {}",
            decoder.packet_id, decoder.length
        );
        let state = clients[client].state;
        if other_packets.is_some() {
            clients[client]
                .connection
                .packets
                .lock()
                .unwrap()
                .push(other_packets.unwrap());
        }
        match state {
            NetworkState::HANDSHAKING => match decoder.packet_id {
                0x00 => {
                    let packet = S00Handshake::decode(decoder);
                    clients.get_mut(client).unwrap().state = packet.next_state;
                }
                _ => Server::unknown_packet(decoder.packet_id),
            },
            NetworkState::STATUS => match decoder.packet_id {
                0x00 => {
                    let json_response = json!({
                        "version": {
                            "name": "1.14.4",
                            "protocol": 498
                        },
                        "players": {
                            "max": 100,
                            "online": 0,
                            "sample": [],
                        },
                        "description": {
                            "text": "Hello World!"
                        }
                    })
                    .to_string();
                    println!("sending response: {}", json_response);
                    let response_encoder = C00Response { json_response }.encode();
                    clients[client].send_packet(response_encoder.finalize(false, None));
                }
                0x01 => {
                    let packet = S01Ping::decode(decoder);
                    let pong_encoder = C01Pong { payload: packet.payload }.encode();
                    clients[client].send_packet(pong_encoder.finalize(false, None));
                },
                _ => Server::unknown_packet(decoder.packet_id),
            },
            NetworkState::LOGIN => match decoder.packet_id {
                0x00 => {}
                _ => Server::unknown_packet(decoder.packet_id),
            },
            NetworkState::PLAY => match decoder.packet_id {
                _ => Server::unknown_packet(decoder.packet_id),
            },
        }
    }

    fn check_incoming_packets(&mut self) {
        let num_clients = self.clients.lock().unwrap().len();
        for client in 0..num_clients {
            let mut packets = self.clients.lock().unwrap()[client]
                .connection
                .receive_packets();
            for packet in packets.drain(..) {
                self.handle_packet(client, packet);
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
    let server = Server::new();
    server.start();
}
