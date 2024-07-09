use std::io::{BufReader, BufWriter};
use std::net::{TcpStream};
use lib::{send_data, receive_data};



fn main() {
    
    let stream = TcpStream::connect("127.0.0.1:4000");
    let stream = match stream {
        Ok(stream) => stream,
        Err(_) => { panic!() } // TODO: log error using pretty_env_logger
    };
    
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    
    let buf = Vec::from("hello".as_bytes());
    
    //send_data(&buf, writer).unwrap()
}
