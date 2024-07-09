use std::io;
use std::io::{Read, Write};



// Function that serializes a byte buffer with a length prefix and sends it through the writer 
pub fn send_data(buf: &[u8], mut writer: impl Write) -> io::Result<()> { // TODO: take in 
    let len_bytes: [u8; 8] = buf.len().to_le_bytes(); 
    // TODO: handle panic if length is too big for array
    // TODO: change length prefix to only 4 bytes, cast usize to u32 somehow
    writer.write_all(&len_bytes)?;
    writer.write_all(buf)?;
    
    writer.flush()
}

pub fn receive_data(mut reader: impl Read) -> Option<Vec<u8>> {
    
    let mut len_bytes = [0u8; 8];
    reader.read_exact(&mut len_bytes).ok()?;
    
    let len = u64::from_le_bytes(len_bytes);
    
    let mut buf: Vec<u8> = Vec::with_capacity(len as usize); // TODO: there's a better way to do this
    
    reader.read_exact(buf.as_mut_slice()).ok()?;
    
    Some(buf)
}