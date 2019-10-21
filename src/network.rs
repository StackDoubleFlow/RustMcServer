extern crate openssl;
extern crate rand;
use crate::packets::*;
use crate::player::Player;
use crate::world::World;
use openssl::pkey::Private;
use openssl::rsa::{Padding, Rsa};
use serde_json::json;
use rand::Rng;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, MutexGuard};
use std::{thread, time};



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
            data.drain(length..);
            data.shrink_to_fit();
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

pub struct Client {
    connection: Connection,
    state: NetworkState,
    pub shared_secret: Option<Vec<u8>>,
    pub compressed: bool,
    verify_token: Option<Vec<u8>>,
    player: Option<Player>,
}

impl Client {
    fn new(stream: TcpStream) -> Client {
        let connection = Connection::new(stream);
        Client {
            connection,
            state: NetworkState::HANDSHAKING,
            shared_secret: None,
            compressed: false,
            verify_token: None,
            player: None,
        }
    }

    fn send_packet(&mut self, encoder: PacketEncoder) {
        let buffer = encoder.finalize(self.compressed, &self.shared_secret);
        self.connection.stream.write(buffer.as_slice()).unwrap();
    }
}

struct Server {
    clients: Arc<Mutex<Vec<Client>>>,
    key_pair: Rsa<Private>,
}

impl Server {
    fn new() -> Server {
        let rsa = Rsa::generate(1024).unwrap();

        Server {
            clients: Arc::new(Mutex::new(Vec::new())),
            key_pair: rsa,
        }
    }

    fn listen_for_connections(&self) {
        let clients = self.clients.clone();
        thread::spawn(move || {
            let listener = TcpListener::bind("0.0.0.0:25566").unwrap();
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
        let (decoder, other_packets) = PacketDecoder::new(packet, &clients[client]);
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
                            "name": "RustMC 1.14.4",
                            "protocol": 498
                        },
                        "players": {
                            "max": 100,
                            "online": 1,
                            "sample": [],
                        },
                        "description": {
                            "text": "Hello World!",
                            "color": "gold"
                        }
                    })
                    .to_string();
                    let response_encoder = C00Response { json_response }.encode();
                    clients[client].send_packet(response_encoder);
                }
                0x01 => {
                    let packet = S01Ping::decode(decoder);
                    let pong_encoder = C01Pong {
                        payload: packet.payload,
                    }
                    .encode();
                    clients[client].send_packet(pong_encoder);
                }
                _ => Server::unknown_packet(decoder.packet_id),
            },
            NetworkState::LOGIN => match decoder.packet_id {
                0x00 => {
                    let packet = S00LoginStart::decode(decoder);
                    let public_key = self.key_pair.public_key_to_der().unwrap();
                    let verify_token = rand::thread_rng().gen::<[u8; 4]>().to_vec();
                    let request_encoder = C01EcryptionRequest {
                        server_id: "".to_string(),
                        public_key_length: public_key.len() as i32,
                        public_key,
                        verify_token_length: 4,
                        verify_token: verify_token.clone(),
                    }.encode();
                    clients[client].verify_token = Some(verify_token);
                    clients[client].send_packet(request_encoder);
                    println!("{}", packet.name);
                }
                0x01 => {
                    let packet = S01EncryptionResponse::decode(decoder);
                    let mut received_verify_token = vec![0u8; self.key_pair.size() as usize];
                    let length = self.key_pair.private_decrypt(packet.verify_token.as_slice(), received_verify_token.as_mut(), Padding::PKCS1).unwrap();
                    received_verify_token.drain(length..received_verify_token.len());
                    if &received_verify_token == clients[client].verify_token.as_ref().unwrap() {
                    } else {
                        for i in 0..4 {
                            print!("{:x}", clients[client].verify_token.as_ref().unwrap()[i]);
                        }
                        println!("");
                        for i in 0..received_verify_token.len() {
                            print!("{:x}", received_verify_token[i]);
                        }
                        println!("");
                    }
                }
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
        self.listen_for_connections();
        loop {
            thread::sleep(time::Duration::from_millis(5));
            self.check_incoming_packets();
        }
    }
}

pub fn start_server() {
    println!("Starting server...");
    let server = Server::new();
    server.start();
}
