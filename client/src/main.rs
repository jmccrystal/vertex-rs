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

// TODO: Implement heartbeat. Automatically attempt reconnect if heartbeat fails.

/// The client will have several actions defined in the lib::Command enum.
/// Each command will correspond to a function to run on the client defined in this impl block.
impl Client {
    fn new(stream: TcpStream) -> Client {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream.try_clone().unwrap());

        Client { reader, writer }
    }

    fn handle_client(&mut self) {
        loop {
            match receive_data(&mut self.reader) {
                Some((command, buf)) => parse_message(self, command, buf),
                None => return,
            };
        }
    }
}

fn main() {
    pretty_env_logger::init();

    let stream = loop {
        match TcpStream::connect(goldberg_string!("127.0.0.1:4000")) {
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

    let mut client = Client::new(stream);

    loop {
        client.handle_client();
    }
}
