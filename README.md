# vertex-rs


## Overview

This project is a remote access trojan with a client-server architecture for multithreaded remote command execution and communication. It consists of a server that can manage multiple client connections and send various commands to be executed on the client machines.

## Components

1. Server
2. Client
3. Shared Library

## Features

- Multi-client support
- Remote command execution (PowerShell commands)
- Echo functionality
- Message popup on client machines
- Screenshot capability (work in progress)
- Client listing
- Error handling and logging

## Usage

### Server

To start the server:

1. Navigate to the server directory
2. Run `cargo run`

The server will start and listen for incoming connections on `127.0.0.1:7878`.

Note: If you want to make the server accessible from other machines:
1. Change the binding address in `main.rs` from `127.0.0.1:7878` to `0.0.0.0:7878`.
2. Ensure port 7878 (or whichever port you choose) is open in your firewall.
3. If you're behind a router, set up port forwarding for port 7878 to your local machine's IP address.

Be aware of the security implications of exposing your server to the network.

### Client

To start a client:

1. Navigate to the client directory
2. Run `cargo run` for debug mode or `cargo run --release` for release mode

The client will attempt to connect to the server at `127.0.0.1:7878`. If the connection fails, it will continuously retry.

Important notes:
- If you're connecting to a server on a different machine, you need to change the IP address in the client's `main.rs` file to the public IP address of the server.
- When compiled in release mode (`cargo run --release` or `cargo build --release`), the client will run silently in the background without opening a console window. This behavior is achieved through the `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` attribute in `main.rs`.

### Commands

Once the server is running and clients are connected, you can use the following commands:

- `echo <ip> <message>`: Send a message to a specific client and receive an echo back
- `echoall <message>`: Echo a message to all connected clients (generally intended for ensuring client connectivity)
- `run <ip> <command>`: Execute a PowerShell command on a specific client
- `popup <ip> <message>`: Display a popup message on a specific client's machine
- `screenshot <ip>`: Attempt to take a screenshot of a specific client's screen (currently unfinished)
- `list`: List all connected client IPs

### Example Usage

```
echo 192.168.1.100 Hello, client!
run 192.168.1.100 Get-Process
popup 192.168.1.100 Your attention is required!
list
```

## Project Structure

- `main.rs`: Server initialization and main loop
- `client.rs`: Client struct and handling
- `command_handler.rs`: Parsing and execution of commands
- `tools.rs`: Implementation of various command functionalities
- `task_manager.rs`: Management of command execution threads
- `error.rs`: Error handling and custom error types
- `types.rs`: Type definitions used across the project

## Development Status

This project is currently in development. Some features, like the screenshot functionality, are not fully implemented or may have issues.

## Future Improvements

- Implement secure authentication and encryption
- Complete and refine the screenshot functionality
- Add more robust error handling and recovery mechanisms
- Implement a more user-friendly interface for the server
- Add support for file transfers between server and clients

## Contributing

Contributions to improve the project are welcome. Please ensure to follow good coding practices and add appropriate tests for new features.

## Disclaimer

This software is provided as-is, without any guarantees or warranties. Use at your own risk.
