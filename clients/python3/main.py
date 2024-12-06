import socket
import struct
import os
import click

SERVER_ADDRESS = ("127.0.0.1", 7878)
MAX_BUFFER_SIZE = 8192
MAX_STATUS_SIZE = 7


cli = click.Group()


def send_command(command, payload=b""):
    with socket.create_connection(SERVER_ADDRESS) as sock:
        sock.sendall(command.encode("utf-8") + payload)

        response = sock.recv(MAX_BUFFER_SIZE).decode("utf-8")

        status = response[:MAX_STATUS_SIZE]

        if status == "*ERROR*":
            raise Exception("Server error, aborting")

        print(f"Server response: {status}")

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

        print(f"Server response: {status}")

        return response


@cli.command()
@click.option("--file-path", type=str, required=True, help="Path to the file to upload")
def upload(file_path):
    if not os.path.isfile(file_path):
        print("File does not exist.")
        return

    file_size = os.path.getsize(file_path)

    payload = struct.pack(">Q", file_size)

    with open(file_path, "rb") as f:
        file_content = f.read()

    payload += file_content

    print(f"Uploading file: {file_path}")

    response = send_command("UPLOAD", payload)

    if response == "UPLOADED":
        print(f"File '{file_path}' uploaded successfully.")
    else:
        print(f"Failed to upload file '{file_path}'.")


@cli.command()
@click.option("--term", type=str, required=True, help="Term to search for")
def search(term):
    payload = struct.pack(">Q", len(term))

    payload += term.encode("utf-8")

    print(f"Searching for term: {term}")

    response = send_command("SEARCH", payload)

    if "SUCCESS" not in response:
        print(f"Term '{term}' not found")
        return

    response = response[MAX_STATUS_SIZE:]

    if response == "":
        print(f"No documents found containing '{term}'")
        return

    print(f"Documents IDs containing '{term}': {response}")


@cli.command()
@click.option(
    "--document-id", type=int, required=True, help="ID of the document to delete"
)
def delete(document_id):
    print(f"Deleting document ID: {document_id}")

    payload = struct.pack(">Q", document_id)

    response = send_command("DELETE", payload)

    if response == "DELETED":
        print(f"Document '{document_id}' deleted successfully.")
    else:
        print(f"Document '{document_id}' not found or could not be deleted.")


@cli.command()
@click.option(
    "--document-id",
    type=int,
    required=True,
    help="ID of the document to download",
)
def download(document_id):
    print(f"Downloading document ID: {document_id}")
    payload = struct.pack(">Q", document_id)

    response = send_command_and_download_bytes("IMPORT", payload)

    if b"SUCCESS" not in response:
        print(f"Document '{document_id}' not found.")
        return

    response = response[MAX_STATUS_SIZE:]
    if len(response) == 0:
        print(f"Document '{document_id}' could not be downloaded.")
        return

    with open(f"document_{document_id}.txt", "wb") as f:
        f.write(response)

    print(f"Document '{document_id}' downloaded successfully.")


if __name__ == "__main__":
    cli()
