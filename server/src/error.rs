use std::fmt;
use serde_json::Error;
use crate::error::CommandErr::*;
use crate::HandleVec;

#[derive(Clone, Debug)]
/// Defines types of errors when sending commands
pub(crate) enum CommandErr {
    ArgNumErr(&'static str),
    SendMessageErr(String, String),
    InvalidCommandErr(&'static str),
    NoClientsErr(&'static str),
    MultipleErr(Vec<CommandErr>),
    DeserializeErr(&'static str),
}

impl CommandErr {
    /// Get inner error message from each error
    pub(crate) fn inner(&self) -> Option<String> {
        let string = match self {
            ArgNumErr(msg) => msg.to_string(),
            SendMessageErr(msg, _) => msg.to_string(),
            InvalidCommandErr(msg) => msg.to_string(),
            NoClientsErr(msg) => msg.to_string(),
            DeserializeErr(msg) => msg.to_string(),
            MultipleErr(_) => return None,
        };
        Some(string)
    }
}

impl From<Error> for CommandErr {
    fn from(_value: Error) -> Self { DeserializeErr("An error occurred while deserializing data") }
}

impl From<std::io::Error> for CommandErr {
    fn from(value: std::io::Error) -> Self {
        DeserializeErr("An I/O error occurred")
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

pub(crate) fn process_err(handles: &HandleVec, err: CommandErr) {
    match err {
        // If there is an error sending message, the client has likely disconnected
        SendMessageErr(msg, ip) => {
            log::error!("{}", msg);
            // Remove offending handle from handles vector
            log::trace!("About to lock in process_err");
            handles.lock().unwrap().retain(|handle| { handle.ip != ip });
            log::trace!("locked in process_err");
            log::info!("Disconnected client with IP {}", ip);
        }
        // If there are multiple errors, process them individually
        MultipleErr(vec) => {
            for err in vec {
                process_err(handles, err);
            }
        },
        // Other errors can be printed
        err => log::error!("{}", err),
    }
}
