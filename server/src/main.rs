mod tools;
mod task_manager;
mod command_handler;
mod client;
mod error;
mod types;
mod banner;

use std::{io, thread};
use std::net::{TcpListener};
use std::sync::{Arc, Mutex};
use crate::banner::print_banner;
use crate::client::Client;
use crate::command_handler::handle_commands;
use crate::types::HandleVec;

fn main() -> io::Result<()> {
    pretty_env_logger::init();

    print_banner();
    
    log::debug!("Connected to stream");
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // Create synchronous vector of handles
    let handles: HandleVec = Arc::new(Mutex::new(Vec::new()));

    // Clone handle vector to use in main thread
    let clone = handles.clone();
    thread::spawn( move || handle_commands(clone));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                log::info!("Found new client with IP {}", stream.peer_addr()?);
                let (mut client, handle) = Client::new(stream);

                // Clone mutex to add handle
                let handles_clone = handles.clone();
                
                handles_clone.lock().unwrap().push(handle);

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