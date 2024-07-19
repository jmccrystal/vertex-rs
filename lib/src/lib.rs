use std::{fmt, io};
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use serde_json::Error;
use crate::CommandErr::*;

static XOR_KEY: &[u8] = b"ENCRYPTION_KEY";


/// Defines the kind of message being sent
#[derive(Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Command {
    Send = 0,
    Echo = 1,
    Run = 2,
    Message = 3,
}


impl Command {
    /// Get a command type from a single byte
    fn from_u8(num: u8) -> Option<Self> {
        match num {
            0 => Some(Command::Send),
            1 => Some(Command::Echo),
            2 => Some(Command::Run),
            3 => Some(Command::Message),
            _ => None,
        }
    }
}


#[derive(Clone)]
/// Defines types of errors when sending commands
pub enum CommandErr {
    ArgNumErr(&'static str),
    SendMessageErr(String),
    InvalidCommandErr(&'static str),
    NoClientsErr(&'static str),
    MultipleErr(Vec<CommandErr>),
    DeserializeErr(&'static str),
}

impl CommandErr {
    /// Get inner error message from each error
    pub fn inner(&self) -> Option<String> {
        let string = match self {
            ArgNumErr(msg) => msg.to_string(),
            SendMessageErr(msg) => msg.to_string(),
            InvalidCommandErr(msg) => msg.to_string(),
            NoClientsErr(msg) => msg.to_string(),
            DeserializeErr(msg) => msg.to_string(),
            MultipleErr(_) => return None,
        };
        Some(string)
    }
}

impl From<serde_json::Error> for CommandErr {
    fn from(_value: Error) -> Self {
        DeserializeErr("An error occurred while deserializing data")
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


/// Function that serializes a byte buffer with a length prefix and sends it through the writer.
/// Structure of data: buf\[0]: type of data. buf[1..5]: length of message. buf[5..]: message as JSON.
pub fn send_data(command: Command, data: &impl Serialize, writer: &mut impl Write) -> io::Result<()> {
    // Encode data into json vec
    let mut buf = serde_json::to_vec(data)?;

    // Make sure message isn't too big for 4 bytes length
    if buf.len() > u32::MAX as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Message length exceeds maximum length"));
    }

    // Encode buffer length to bytes
    let len_bytes: [u8; 4] = (buf.len() as u32).to_le_bytes();

    // Encrypt buffer
    xor(&mut buf, XOR_KEY);

    // Write all data to stream
    writer.write_all(&[command as u8])?;
    writer.write_all(&len_bytes)?;
    writer.write_all(&buf)?;

    writer.flush()
}


pub fn receive_data(reader: &mut impl Read) -> Option<(Command, Vec<u8>)> {
    // Read command type
    let mut command = [0u8; 1];
    reader.read_exact(&mut command).ok()?;
    let command = Command::from_u8(command[0])?;

    // Read length of message
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes).ok()?;
    let len = u32::from_le_bytes(len_bytes);

    // Read buffer
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).ok()?;

    // Decrypt data
    xor(&mut buf, XOR_KEY);

    Some((command, buf))
}


/// Encrypts or decrypts the message in `buf` by XORing each index by the same index in `key` (mod key.len())
fn xor(buf: &mut [u8], key: &[u8]) {
    for (i, n) in buf.iter_mut().enumerate() {
        *n ^= key[i % key.len()];
    }
}


#[cfg(test)]
mod tests {
    use crate::xor;
    #[test]
    fn test_xor_encrypt() {
        const TEST_STR: &[u8] = b"Hello World!";
        let mut buf = Vec::new();
        for n in TEST_STR {
            buf.push(*n)
        }
        let key = b"Key";

        xor(&mut buf, key);
        println!("Encrypted: {:#?}", buf);
        xor(&mut buf, key);
        println!("Decrypted: {:#?}", buf);

        assert_eq!(buf, TEST_STR);
    }
}