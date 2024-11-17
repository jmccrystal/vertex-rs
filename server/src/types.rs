use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

use lib::Command;

use crate::client::ClientHandle;

pub(crate) type HandleVec = Arc<Mutex<Vec<ClientHandle>>>;

pub(crate) type ClientHandleSender = Sender<(Command, String, Sender<Option<Vec<u8>>>)>;
pub(crate) type ClientHandleReceiver = Receiver<(Command, String, Sender<Option<Vec<u8>>>)>;