import socket
import struct
import os

SERVER_ADDRESS = ("127.0.0.1", 7878)
MAX_BUFFER_SIZE = 8192
MAX_STATUS_SIZE = 7


def send_command(command, payload=b""):
    with socket.create_connection(SERVER_ADDRESS) as sock:
        sock.sendall(command.encode("utf-8"))
        sock.sendall(payload)

        response = sock.recv(MAX_BUFFER_SIZE).decode("utf-8")

        status = response[:MAX_STATUS_SIZE]

        if status == "*ERROR*":
            raise Exception("Server error, aborting")

        return response


def send_command_and_download_bytes(command, payload=b"") -> bytes:
    with socket.create_connection(SERVER_ADDRESS) as sock:
        sock.sendall(command.encode("utf-8") + payload)

        response = bytes()

        while True:
            data = sock.recv(MAX_BUFFER_SIZE)
            if not data:
                break
            response += data

        status = response[:MAX_STATUS_SIZE]

        if status == "*ERROR*":
            raise Exception("Server error, aborting")

        return response


def upload_file(file_path):
    if not os.path.isfile(file_path):
        print("File does not exist.")
        return

    with open(file_path, "rb") as f:
        file_content = f.read()

    payload = struct.pack(">Q", len(file_content))

    payload += file_content

    return send_command("UPLOAD", payload)


def get_document_count():
    response = send_command_and_download_bytes("STATUS")

    if b"SUCCESS" not in response:
        print("Server is not available.")
        return

    response = response[MAX_STATUS_SIZE:]
    if len(response) != 8:
        print("Server is not in a ready state.")
        return

    return struct.unpack(">Q", response)[0]
