use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{TcpStream};
use std::str::from_utf8;
use lib::{send_data, receive_data};


fn echo(reader: &mut impl Read, writer: &mut impl Write) {
    loop {
        if let Some(received_bytes) = receive_data(reader) {
            log::debug!("Received bytes: {:?}. As string: \"{}\"", received_bytes, from_utf8(&received_bytes).unwrap());
            send_data(&received_bytes, writer).unwrap();
        }
    }
}

fn main() {

    pretty_env_logger::init();

    let stream = loop {
        match TcpStream::connect("127.0.0.1:4000") {
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

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = BufWriter::new(stream.try_clone().unwrap());

    echo(&mut reader, &mut writer);
}
