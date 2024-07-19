use std::sync::{Arc, Mutex};
use lib::{Command, CommandErr};
use crate::ClientHandle;
use lib::CommandErr::*;



pub fn echo(args: Vec<&str>, handles: &Arc<Mutex<Vec<ClientHandle>>>) -> Result<String, CommandErr> {
    if args.len() <= 2 {
        return Err(ArgNumErr("Incorrect number of arguments. Usage: echo [IP] [MESSAGE]"))
    }

    let ip = args[1].to_string();
    let message = args[2..].join(" ");

    for handle in handles.lock().unwrap().iter() {
        if handle.ip == ip {
            if let Some(buf) = handle.send_to_client(Command::Echo, message.clone()) {
                let response: String = serde_json::from_slice(&buf)?;
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
        if let Some(buf) = handle.send_to_client(Command::Echo, message.clone()) {
            let response: String = serde_json::from_slice(&buf)?;
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
    if args.len() <= 2 {
        return Err(ArgNumErr("Incorrect number of arguments. Usage: run [IP] [COMMAND]"))
    }
    let ip = args[1];

    // Join every argument past the IP into a single command to run
    let command = args[2..].join(" ");

    log::trace!("Sending command: {}", command);
    for handle in handles.lock().unwrap().iter() {
        if handle.ip == ip {
            if let Some(buf) = handle.send_to_client(Command::Run, command.clone()) {
                let response: String = serde_json::from_slice(&buf)?;
                return Ok(format!("Successfully sent command {} to client with IP {}. Response: {}", command, handle.ip, response))
            }
            else {
                return Err(SendMessageErr(format!("An error occurred while sending command {} to client with IP {}", command, handle.ip)))
            }
        }
    }
    Err(NoClientsErr("No clients exist"))
}

pub fn popup(args: Vec<&str>, handles: &Arc<Mutex<Vec<ClientHandle>>>) -> Result<String, CommandErr> {
    if args.len() <= 2 {
        return Err(ArgNumErr("Incorrect number of arguments. Usage: popup [IP] [MESSAGE]"))
    }

    let ip = args[1];
    let message = args[2..].join(" ");

    for handle in handles.lock().unwrap().iter() {
        if handle.ip == ip {
            if handle.send_to_client(Command::Message, message.clone()).is_some() {
                return Ok(format!("Successfully sent popup with message {} to client with IP {}.", message, handle.ip))
            }
            else {
                return Err(SendMessageErr(format!("An error occurred while sending command {} to client with IP {}", message, handle.ip)))
            }
        }
    }
    Err(NoClientsErr("No clients exist"))
}