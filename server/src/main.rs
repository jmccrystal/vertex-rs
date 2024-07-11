use std::{io, thread};
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender, SendError};
use lib::{send_data, receive_data};


// #[repr(u8)]
// enum Command {
//     Send = 0,
//     Echo = 1,
//     Run = 2,
// }

#[derive(Clone)]
struct ClientHandle {
    sender: Sender<String>,
    ip: String,
}

impl ClientHandle {

    // Sends a message to the corresponding Client object
    fn send_message(&self, message: String) -> Result<(), SendError<String>> {
        self.sender.send(message)
    }
}


struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    receiver: Receiver<String>,
}


impl Client {
    fn new(stream: TcpStream) -> (Client, ClientHandle) {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream.try_clone().unwrap());

        let ip = stream.peer_addr().unwrap().to_string();

        let (sender, receiver) = channel();

        (Client { reader, writer, receiver }, ClientHandle { sender, ip })
    }

    fn send_message(&mut self) -> Option<String> {
        let message = self.receiver.recv().ok()?;

        if send_data(message.as_bytes(), &mut self.writer).is_ok() {
            if let Some(bytes) = receive_data(&mut self.reader) {
                let response = String::from_utf8_lossy(&bytes);
                Some(response.parse().unwrap())
            }
            else { None }
        }
        else { None }
    }
    fn handle_client(&mut self) {
        loop {
            if let Some(response) = self.send_message() {
                log::info!("Successful response: {}", response);
            }
            else {
                log::error!("An error occurred while receiving response");
            }

        }
    }
}

fn handle_clients(handles: Arc<Mutex<Vec<ClientHandle>>>) {

    let mut input = String::new();
    loop {
        io::stdin().read_line(&mut input).unwrap();
        let split = input.trim().split(' ').collect::<Vec<&str>>();

        let command = split[0];

        let handles = handles.clone();


        // Usage: echo (ip) (message)
        if command == "echo" {
            
            if split.len() != 3 {
                log::error!("Incorrect number of arguments");
                input.clear();
                continue;
            }
            
            let ip = split[1].to_string();
            let message = split[2].to_string();

            for handle in handles.lock().unwrap().iter() {
                if handle.ip == ip {
                    if handle.send_message(message.clone()).is_ok() {
                        log::info!("Successfully sent message {} to client with IP {}", message, ip);
                    }
                    else {
                        log::error!("An error occurred while sending message {} to client with IP {}", message, ip);
                    }
                }
            }
        }
        // Usage: echoall (message)
        else if command == "echoall" {
            if split.len() != 2 {
                log::error!("Incorrect number of arguments");
                input.clear();
                continue;
            }
            
            let message = split[1].to_string();

            for handle in handles.lock().unwrap().iter() {
                if handle.send_message(message.clone()).is_ok() {
                    log::info!("Successfully sent message {} to client with IP {}", message, handle.ip);
                }
                else {
                    log::error!("An error occurred while sending message {} to client with IP {}", message, handle.ip)
                }
            }
        }
        else {
            log::error!("Please enter a valid command");
        }

        input.clear();
        drop(handles);
    }
}


fn main() -> io::Result<()> {

    pretty_env_logger::init();
    log::debug!("Connected to stream");
    let listener = TcpListener::bind("127.0.0.1:4000")?;

    // Create synchronous vector of handles
    let handles: Arc<Mutex<Vec<ClientHandle>>> = Arc::new(Mutex::new(Vec::new()));

    // Clone handle vector to use in main thread
    let clone = handles.clone();
    thread::spawn( move || handle_clients(clone));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                log::info!("Found new client with IP {}", stream.peer_addr().unwrap());
                let (mut client, handle) = Client::new(stream);

                // Clone handle to be added to handles vector
                let handle_clone = handle.clone();

                // Clone mutex to add handle
                let handles_clone = handles.clone();
                handles_clone.lock().unwrap().push(handle_clone);

                // Run main logic on each client
                thread::spawn( move || client.handle_client());
            }
            Err(err) => {
                log::error!("Error while parsing stream: {}", err);
            }
        }
    }

    Ok(())
}