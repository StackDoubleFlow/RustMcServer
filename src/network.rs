extern crate openssl;
extern crate rand;
extern crate reqwest;
use crate::mojang;
use crate::mojang::{Mojang, MojangHasJoinedResponse};
use crate::packets::*;
use crate::player::Player;
use openssl::pkey::Private;
use openssl::rsa::{Padding, Rsa};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::yield_now;

struct Connection {
    packet_receiver: mpsc::Receiver<PacketBuffer>,
    stream: TcpStream,
    alive: bool,
}

impl Connection {
    fn new(stream: TcpStream) -> Connection {
        println!("New connection!");
        let reader = stream.try_clone().unwrap();
        let (tx, rx) = mpsc::channel();
        let connection = Connection {
            packet_receiver: rx,
            stream,
            alive: true,
        };

        thread::spawn(|| {
            Connection::handle_connection(reader, tx);
        });
        connection
    }

    fn handle_connection(mut stream: TcpStream, packet_sender: mpsc::Sender<PacketBuffer>) {
        loop {
            let mut data = vec![0u8; 512];
            let length = stream.read(&mut data).unwrap();
            if length == 0 {
                continue;
            }
            data.shrink_to_fit();
            packet_sender.send(data);
        }
    }

    fn receive_packets(&mut self) -> Vec<PacketBuffer> {
        let mut packets = Vec::new();
        loop {
            match self.packet_receiver.try_recv() {
                Ok(packet) => packets.push(packet),
                Err(mpsc::TryRecvError::Empty) => return packets,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.alive = false;
                    return packets;
                }
            }
        }
    }
}

pub struct Client {
    connection: Connection,
    state: NetworkState,
    pub shared_secret: Option<Vec<u8>>,
    pub compressed: bool,
    verify_token: Option<Vec<u8>>,
    player: Option<Player>,
    username: Option<String>,
    id: u32,
}

impl Client {
    fn new(stream: TcpStream, id: u32) -> Client {
        let connection = Connection::new(stream);
        Client {
            connection,
            state: NetworkState::HANDSHAKING,
            shared_secret: None,
            compressed: false,
            verify_token: None,
            player: None,
            username: None,
            id,
        }
    }

    fn send_packet(&mut self, encoder: PacketEncoder) {
        let buffer = encoder.finalize(self.compressed, &self.shared_secret);
        self.connection.stream.write(buffer.as_slice()).unwrap();
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    maxPlayers: i32,
    motd: String,
}

pub struct Server {
    clients: Vec<Client>,
    client_receiver: mpsc::Receiver<Client>,
    key_pair: Rsa<Private>,
    mojang: Mojang,
}

impl Server {
    fn new() -> Server {
        let rsa = Rsa::generate(1024).unwrap();
        let (tx, rx) = mpsc::channel();
        let server = Server {
            clients: Vec::new(),
            key_pair: rsa,
            mojang: Mojang::new(),
            client_receiver: rx
        };
        server.listen_for_connections(tx);
        server
    }

    fn get_client(&self, clientId: u32) -> &Client {
        self.clients.iter().filter(|client| client.id == clientId).collect::<Vec<&Client>>()[0]
    }

    fn listen_for_connections(&self, sender: mpsc::Sender<Client>) {
        let mut next_id = 0;
        thread::spawn(move || {
            let listener = TcpListener::bind("0.0.0.0:25566").unwrap();
            for stream in listener.incoming() {
                let stream = stream.unwrap();

                let client = Client::new(stream, next_id);
                sender.send(client).unwrap();
                next_id += 1;
            }
        });
    }

    fn unknown_packet(id: i32) {
        eprintln!("Unknown packet with id: {}", id);
    }

    fn handle_packet(&mut self, client: usize, packet: PacketBuffer) {
        let decoder = PacketDecoder::new(packet, &self.clients[client]);
        println!(
            "Packet received: {}, with the length of: {}",
            decoder.packet_id, decoder.length
        );
        let state = self.clients[client].state;
        match state {
            NetworkState::HANDSHAKING => match decoder.packet_id {
                0x00 => {
                    let packet = S00Handshake::decode(decoder);
                    println!("New state: {:#?}", packet.next_state);
                    self.clients.get_mut(client).unwrap().state = packet.next_state;

                    if packet.next_state == NetworkState::STATUS {
                        let json_response = json!({
                            "version": {
                                "name": "RustMC 1.15.1",
                                "protocol": 575
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
                        self.clients[client].send_packet(response_encoder);
                    }
                }
                _ => Server::unknown_packet(decoder.packet_id),
            },
            NetworkState::STATUS => match decoder.packet_id {
                0x00 => {
                    let json_response = json!({
                        "version": {
                            "name": "RustMC 1.15.1",
                            "protocol": 575
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
                    self.clients[client].send_packet(response_encoder);
                }
                0x01 => {
                    let packet = S01Ping::decode(decoder);
                    let pong_encoder = C01Pong {
                        payload: packet.payload,
                    }
                    .encode();
                    self.clients[client].send_packet(pong_encoder);
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
                    }
                    .encode();
                    self.clients[client].verify_token = Some(verify_token);
                    self.clients[client].username = Some(packet.name);
                    self.clients[client].send_packet(request_encoder);
                }
                0x01 => {
                    let packet = S01EncryptionResponse::decode(decoder);
                    let mut received_verify_token = vec![0u8; packet.verify_token_length as usize];

                    let length_decrypted = self
                        .key_pair
                        .private_decrypt(
                            packet.verify_token.as_slice(),
                            received_verify_token.as_mut(),
                            Padding::PKCS1,
                        )
                        .unwrap();
                    received_verify_token.drain(length_decrypted..received_verify_token.len());
                    if &received_verify_token == self.clients[client].verify_token.as_ref().unwrap() {
                        // Start login process
                        /*self.mojang.send_has_joined(
                            &clients[client].username.unwrap(),
                            clients[client].id
                        );*/
                    } else {
                        println!("Verify token incorrent!!");
                    }
                }
                _ => Server::unknown_packet(decoder.packet_id),
            },
            NetworkState::PLAY => match decoder.packet_id {
                _ => Server::unknown_packet(decoder.packet_id),
            },
        }
    }

    fn on_mojang_has_joined_response(&mut self, client_id: u32, result: MojangHasJoinedResponse) {
        let client = self.get_client(client_id);

    }

    fn receive_packets(&mut self) {
        let num_clients = self.clients.len();
        for client in 0..num_clients {
            let mut packets = self.clients[client]
                .connection
                .receive_packets();
            for packet_batch in packets.drain(..) {
                for packet in PacketDecoder::newBatch(packet_batch, &self.clients[client]) {
                    self.handle_packet(client, packet.buffer);
                }
            }
        }
    }

    fn receive_clients(&mut self) {
        let result = self.client_receiver.try_recv();
        if let Ok(client) = result {
            self.clients.push(client);
        }
    }

    fn poll_mojang(&mut self) { // TODO: Clean up maybe
        let mut finished_indicies = Vec::new();
        for (i, pending) in self.mojang.hasJoinedPending.iter().enumerate() {
            if pending.result.is_some() {
                finished_indicies.push(i);
            }
        }
        for index in finished_indicies {
            let response = self.mojang.hasJoinedPending.remove(index);
            self.on_mojang_has_joined_response(response.clientId, response.result.unwrap());
        }
        self.mojang.clean();
    }

    fn start(mut self) {
        println!("Listening for connections...");
        loop {
            std::thread::sleep(std::time::Duration::from_millis(5));
            yield_now();
            self.receive_clients();
            self.receive_packets();
            self.poll_mojang();
        }
    }
}

pub fn start_server() {
    println!("Starting server...");
    let server = Server::new();
    server.start();
}
