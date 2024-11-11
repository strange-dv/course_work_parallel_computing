import socket
import struct
import os
import click

SERVER_ADDRESS = ("127.0.0.1", 7878)


cli = click.Group()


def send_command(command, payload=b""):

    with socket.create_connection(SERVER_ADDRESS) as sock:
        sock.sendall(command.encode("utf-8") + payload)

        response = sock.recv(1024).decode("utf-8")

        print(f"Server response: {response}")

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

    if "FOUND" not in response:
        print(f"Term '{term}' not found")
        return

    response = response[5:]

    if response == "":
        print(f"No documents found containing '{term}'")
        return

    documents = list(map(lambda x: int(x), response.split(",")))

    print(f"Documents IDs containing '{term}': {documents}")


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


if __name__ == "__main__":
    cli()

    upload("~/kpi/year4/parallel_computing/aclImdb/test/neg/0_2.txt")

    search("response")

    delete(0)
