
extern crate openssl;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;


use openssl::rsa::{Rsa, Padding};


use crate::packets::*;
use crate::world::World;






struct Connection {
    packets: Arc<Mutex<Vec<PacketBuffer>>>,
    stream: TcpStream
}

impl Connection {

    fn new(mut stream: TcpStream) -> Connection {
        println!("New connection!");
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let connection = Connection {
            packets: Arc::new(Mutex::new(Vec::new())),
            stream
        };

        let packets_clone = connection.packets.clone();

        thread::spawn(|| {
            Connection::handle_connection(reader, packets_clone);
        });
        connection
    }

    fn handle_connection(mut reader: BufReader<TcpStream>, packets: Arc<Mutex<Vec<PacketBuffer>>>) {
        loop {
            let mut data = vec![0u8; 512];
            let length = reader.read(&mut data).unwrap();
            data.drain(0..length);
            data.shrink_to_fit();
            packets.lock().unwrap().push(data);
        }
    }
}

struct Client {
    connection: Connection
}

impl Client {
    fn new(stream: TcpStream) -> Client {
        let connection = Connection::new(stream);
        Client {
            connection
        }
    }
}

fn handle_packets() {

}

struct Server {
    clients: Arc<Mutex<Vec<Client>>>
}

impl Server {
    fn new() -> Server {
        Server {
            clients: Arc::new(Mutex::new(Vec::new()))
        }
    }
    
    fn listen_for_connections(&self) {
        let clients = self.clients.clone();
        thread::spawn(move || {
            let listener = TcpListener::bind("0.0.0.0:25566").unwrap();
            for stream in listener.incoming() {
                let stream = stream.unwrap();

                let mut client = Client::new(stream);
                clients.lock().unwrap().push(client);    
            }
        });
    }

    fn start(mut self) {
        let rsa = Rsa::generate(1024).unwrap();
        
    }
}

pub fn start_server() {
    
    let mut server = Server::new();
    server.start();
}