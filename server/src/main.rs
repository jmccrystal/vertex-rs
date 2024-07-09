use std::io;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use lib::{send_data, receive_data};

fn handle_client(stream: TcpStream) -> io::Result<()> {
    let response = receive_data(BufReader::new(stream));

    dbg!(response);
    
    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000")?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream).unwrap()
            }
            Err(err) => {
                eprintln!("Error while parsing stream: {}", err);
            }
        }
    }



    Ok(())
}