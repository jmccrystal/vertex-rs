use std::fmt;
use std::sync::{Arc, Mutex};
use lib::Command;
use crate::ClientHandle;
use crate::tools::CommandErr::*;



#[derive(Clone)]
pub enum CommandErr {
    ArgNumErr(&'static str),
    SendMessageErr(String),
    InvalidCommandErr(&'static str),
    NoClientsErr(&'static str),
    MultipleErr(Vec<CommandErr>),
}

impl CommandErr {
    pub fn inner(&self) -> Option<String> {

        let string = match self {
            ArgNumErr(msg) => msg.to_string(),
            SendMessageErr(msg) => msg.to_string(),
            InvalidCommandErr(msg) => msg.to_string(),
            NoClientsErr(msg) => msg.to_string(),
            MultipleErr(_) => return None,
        };
        Some(string)
    }
}

impl fmt::Display for CommandErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.inner().is_none() {
            return Ok(())
        }
        write!(f, "{}", self.inner().unwrap())
    }
}

pub fn echo(args: Vec<&str>, handles: &Arc<Mutex<Vec<ClientHandle>>>) -> Result<String, CommandErr> {
    if args.len() != 3 {
        return Err(ArgNumErr("Incorrect number of arguments. Usage: echo [IP] [MESSAGE]"))
    }

    let ip = args[1].to_string();
    let message = args[2..].join(" ");

    for handle in handles.lock().unwrap().iter() {
        if handle.ip == ip {
            if let Some(response) = handle.send_to_client(Command::Echo, message.clone()) {
                // log::info!;
                return Ok(format!("Successfully echoed message {} to client with IP {}. Response: {}", message, ip, response));
            } else {
                return Err(SendMessageErr(format!("An error occurred while sending message {} to client with IP {}", message, ip)));
            }
        }
    }
    Err(NoClientsErr("No clients exist"))
}

pub fn echoall(args: Vec<&str>, handles: &Arc<Mutex<Vec<ClientHandle>>>) -> Result<String, CommandErr> {
    if args.len() == 1 {
        return Err(ArgNumErr("Incorrect number of arguments. Usage: echoall [MESSAGE]"))
    }
    
    let mut echo_attempt = false;

    let message = args[1..].join(" ");

    let mut error_vec: Vec<CommandErr> = Vec::new();
    
    for handle in handles.lock().unwrap().iter() {
        if let Some(response) = handle.send_to_client(Command::Echo, message.clone()) {
            log::info!("Successfully echoed message {} to client with IP {}. Response: {}", message, handle.ip, response);
            echo_attempt = true;
        } else {
            error_vec.push(SendMessageErr(format!("An error occurred while sending message {} to client with IP {}", message, handle.ip)));
            echo_attempt = true;
            continue
        }
    }

    // Check if any clients exist
    if !echo_attempt {
        return Err(NoClientsErr("No clients exist"));
    }
    
    match error_vec.len() {
        0 => Ok(format!("Successfully sent message {} to all clients", message)),
        1 => Err(error_vec[0].clone()),
        _ => Err(MultipleErr(error_vec)),
    }
    
}

pub fn run(args: Vec<&str>, handles: &Arc<Mutex<Vec<ClientHandle>>>) -> Result<String, CommandErr> {
    if args.len() == 1 || args.len() == 2 {
        return Err(ArgNumErr("Incorrect number of arguments. Usage: run [IP] [COMMAND]"))
    }
    let ip = args[1];

    // Join every argument past the IP into a single command to run
    let command = args[2..].join(" ");

    log::trace!("Sending command: {}", command);
    for handle in handles.lock().unwrap().iter() {
        if handle.ip == ip {
            if let Some(response) = handle.send_to_client(Command::Run, command.clone()) {
                return Ok(format!("Successfully sent command {} to client with IP {}. Response: {}", command, handle.ip, response))
            }
            else {
                return Err(SendMessageErr(format!("An error occurred while sending command {} to client with IP {}", command, handle.ip)))
            }
        }
    }
    Err(NoClientsErr("No clients exist"))
}
