import subprocess
import sys
import time
import socket
from threading import Thread
from queue import Queue

def reader_to_queue(stream, q):
    for line in iter(stream.readline, b''):
        line_str = line.decode('utf-8', errors='replace').strip()
        q.put(line_str)
    stream.close()

def main():
    # Start the server as a subprocess
    server = subprocess.Popen(
        ["target/release/server.exe"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=False,  # Binary mode
    )

    # Queue to hold server output lines
    output_queue = Queue()

    # Spawn threads to read stdout & stderr lines
    t_out = Thread(target=reader_to_queue, args=(server.stdout, output_queue), daemon=True)
    t_err = Thread(target=reader_to_queue, args=(server.stderr, output_queue), daemon=True)
    t_out.start()
    t_err.start()

    client_count = 0

    try:
        while True:
            # Connect a new client
            try:
                s = socket.create_connection(("127.0.0.1", 8080), timeout=1)
            except Exception:
                print(f"Failed to connect new client at count {client_count}", file=sys.stderr)
                break
            client_count += 1
            print(client_count)

            # Send echoall command
            cmd = "echoall asdf\n"
            server.stdin.write(cmd.encode('utf-8'))
            server.stdin.flush()

            # Check server output lines for "disconnected"
            found_error = False
            while True:
                try:
                    line = output_queue.get_nowait()
                    if "disconnected" in line.lower():
                        found_error = True
                        break
                except:
                    # No more lines at the moment
                    break

            if found_error:
                print(f"Server error encountered after {client_count} clients", file=sys.stderr)
                break

    finally:
        # Attempt to terminate server
        server.terminate()
        server.wait(timeout=5)

    print(f"Max clients before failure: {client_count}")

if __name__ == "__main__":
    main()
