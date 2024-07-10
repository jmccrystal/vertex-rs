use std::io;
use std::io::{Read, Write};



// Function that serializes a byte buffer with a length prefix and sends it through the writer
// Structure of data: buf[0]: type of data. buf[1..5]: length of message. buf[5..]: message as bytes.
pub fn send_data(buf: &[u8], writer: &mut impl Write) -> io::Result<()> {

    if buf.len() > u32::MAX as usize { // Make sure message isn't too big for 4 bytes length
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Message length exceeds maximum length"));
    }

    let len_bytes: [u8; 4] = (buf.len() as u32).to_le_bytes();

    // writer.write_all(&[data_type])?;
    writer.write_all(&len_bytes)?;
    writer.write_all(buf)?;
    
    writer.flush()
}

pub fn receive_data(reader: &mut impl Read) -> Option<Vec<u8>> {
    
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes).ok()?;
    
    let len = u32::from_le_bytes(len_bytes);
    
    let mut buf = vec![0u8; len as usize];
    
    reader.read_exact(&mut buf).ok()?;
    
    Some(buf)
}