use std::sync::{Arc, Mutex};
use lib::Command;
use crate::ClientHandle;
use crate::tools::CommandErr::*;



#[derive(Clone)]
pub enum CommandErr {
    ArgNumErr(&'static str),
    SendMessageErr(String),
    UnknownErr(&'static str),
    InvalidCommandErr(&'static str),
    MultipleErr(Vec<CommandErr>),
}

impl CommandErr {
    pub fn inner(self) -> Option<String> {
        let string = match self {
            ArgNumErr(msg) => msg.to_string(),
            SendMessageErr(msg) => msg,
            UnknownErr(msg) => msg.to_string(),
            InvalidCommandErr(msg) => msg.to_string(),
            MultipleErr(_) => return None,
        };
        Some(string)
    }
}

pub fn echo(args: Vec<&str>, handles: &Arc<Mutex<Vec<ClientHandle>>>) -> Result<String, CommandErr> {
    if args.len() != 3 {
        return Err(ArgNumErr("Incorrect number of arguments. Usage: echo [IP] [MESSAGE]"))
    }

    let ip = args[1].to_string();
    let message = args[2].to_string();

    for handle in handles.lock().unwrap().iter() {
        if handle.ip == ip {
            if handle.send_message(Command::Echo, message.clone()).is_ok() {
                // log::info!;
                return Ok(format!("Successfully sent message {} to client with IP {}", message, ip))
            } else {
                // log::error!("An error occurred while sending message {} to client with IP {}", message, ip); TODO: pass off message and ip into string literal and return in the Err
                return Err(SendMessageErr(format!("An error occurred while sending message {} to client with IP {}", message, ip)))
            }
        }
    }
    Err(UnknownErr("An unknown error has occurred"))
}

pub fn echoall(args: Vec<&str>, handles: &Arc<Mutex<Vec<ClientHandle>>>) -> Result<String, CommandErr> {
    if args.len() != 2 {
        return Err(ArgNumErr("Incorrect number of arguments. Usage: echoall [MESSAGE]"))
    }

    let message = args[1].to_string();

    let mut error_vec: Vec<CommandErr> = Vec::new();
    
    for handle in handles.lock().unwrap().iter() {
        if handle.send_message(Command::Echo, message.clone()).is_err() {
            error_vec.push(SendMessageErr(format!("An error occurred while sending message {} to client with IP {}", message, handle.ip)));
            continue
        }
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
            return if handle.send_message(Command::Run, command.clone()).is_ok() {
                Ok(format!("Successfully sent {} to client with IP {}", command, handle.ip))
            }
            else {
                Err(SendMessageErr(format!("An error occurred while sending command {} to client with IP {}", command, handle.ip)))
            }
        }
    }
    Err(UnknownErr("An unknown error has occurred"))
}
