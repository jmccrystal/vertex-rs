mod tools;

use std::{io, thread};
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender, SendError};
use lib::{send_data, receive_data, Command};
use crate::tools::{echo, echoall, run};
use crate::tools::CommandErr::*;

#[derive(Clone)]
struct ClientHandle {
    sender: Sender<(Command, String)>,
    ip: String,
}

impl ClientHandle {
    // Sends a message to the corresponding Client object
    fn send_message(&self, command: Command, message: String) -> Result<(), SendError<(Command, String)>> {
        self.sender.send((command, message))
    }
}


struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    receiver: Receiver<(Command, String)>,
    ip: String,
}


impl Client {
    fn new(stream: TcpStream) -> (Client, ClientHandle) {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream.try_clone().unwrap());

        let ip = stream.peer_addr().unwrap().to_string();

        let (sender, receiver) = channel();

        (Client { reader, writer, receiver, ip: ip.clone() }, ClientHandle { sender, ip })
    }

    fn send_message(&mut self) -> Option<String> {
        let (command, message) = self.receiver.recv().ok()?;

        if send_data(command as u8, message.as_bytes(), &mut self.writer).is_ok() {
            if let Some((Command::Send, bytes)) = receive_data(&mut self.reader) {
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
                log::info!("Successful response from client with IP {}: {}", self.ip, response);
            }
            else {
                log::error!("An error occurred while receiving response from IP {}", self.ip);
            }

        }
    }
}

fn handle_clients(handles: Arc<Mutex<Vec<ClientHandle>>>) {
    let mut input = String::new();
    loop {
        io::stdin().read_line(&mut input).unwrap();
        let args = input.trim().split(' ').collect::<Vec<&str>>();

        let command = args[0];

        let handles = handles.clone();


        let error = match command {
            "echo" => echo(args, &handles),
            "echoall" => echoall(args, &handles),
            "run" => run(args, &handles),
            _ => Err(InvalidCommandErr("The command specified does not exist")),
        };
        
        match error {
            Ok(msg) => log::info!("{}", msg),
            Err(MultipleErr(vec)) => vec.iter().for_each(|err| log::error!("{}", err.clone().inner().unwrap())),
            Err(err) => log::error!("{}", err.inner().unwrap()),
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
    thread::spawn(move || handle_clients(clone));

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