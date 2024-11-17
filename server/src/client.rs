use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use lib::{Command, receive_data, send_data};
use crate::types::{ClientHandleReceiver, ClientHandleSender};

#[derive(Clone)]
pub(crate) struct ClientHandle {
    sender: ClientHandleSender,
    pub(crate) ip: String,
}

impl ClientHandle {
    // TODO: send_to_client should take a generic message, not just String
    /// Sends a message to the corresponding Client object
    pub(crate) fn send_to_client(&self, command: Command, message: String) -> Option<Vec<u8>> {
        // Create new channel to receive response from client
        let (client_sender, handle_receiver) = channel();

        // Should be safe to unwrap since there should always be a corresponding receiver
        self.sender.send((command, message, client_sender)).unwrap();

        // Receive response from client. Client will always send, so unwrap is fine
        handle_receiver.recv().unwrap()
    }
}

pub(crate) struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    receiver: ClientHandleReceiver,
}

impl Client {
    pub(crate) fn new(stream: TcpStream) -> (Client, ClientHandle) {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream.try_clone().unwrap());

        let ip = stream.peer_addr().unwrap().to_string();

        let (sender, receiver) = channel();

        (Client { reader, writer, receiver, }, ClientHandle { sender, ip, })
    }

    /// Waits to receive data from ClientHandle, sends response back once received from client
    fn send_message(&mut self) -> Result<(), ()> {
        let mut response_to_send = None;

        let (command, message, sender) = match self.receiver.recv() {
            Ok((command, message, sender)) => (command, message, sender),
            Err(_) => return Err(()),
        };

        if send_data(command, &message, &mut self.writer).is_ok() {
            // Client should only use Command::Send to send back a response
            if let Some((Command::Send, buf)) = receive_data(&mut self.reader) {
                response_to_send = Some(buf);
            }
        }  
        sender.send(response_to_send).map_err(|_| ())
    }
    pub(crate) fn handle_client(&mut self) {
        loop {
            // Be ready to send another message, or drop client if error occurs
            match self.send_message() {
                Ok(_) => continue,
                Err(_) => return,
            }
        }
    }
}
