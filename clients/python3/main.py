import struct
import os
import click
from load_testing import LoadTester
from send import (
    get_document_count,
    send_command,
    send_command_and_download_bytes,
    upload_file,
    MAX_STATUS_SIZE,
)


cli = click.Group()


@cli.command()
@click.option("--file-path", type=str, required=True, help="Path to the file to upload")
def upload(file_path):
    print(f"Uploading file: {file_path}")

    response = upload_file(file_path)

    if response == "SUCCESS":
        print(f"File '{file_path}' uploaded successfully.")
    else:
        print(f"Failed to upload file '{file_path}'.")


@cli.command()
@click.option("--term", type=str, required=True, help="Term to search for")
def search(term):
    payload = struct.pack(">Q", len(term))

    payload += term.encode("utf-8")

    print(f"Searching for term: {term}")

    response = send_command_and_download_bytes("SEARCH", payload)

    if b"SUCCESS" != response[:MAX_STATUS_SIZE]:
        print(f"Term '{term}' not found")
        return

    response = response[MAX_STATUS_SIZE:].decode("utf-8")

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


@cli.command()
@click.option(
    "--num-threads",
    type=int,
    required=True,
    help="Number of threads to use for load testing",
)
@click.option(
    "--data-dir",
    type=str,
    required=True,
    help="Directory containing the data files",
)
def load(num_threads: int, data_dir: str):
    print("Performing load testing")

    load_tester = LoadTester(num_threads, data_dir)

    load_tester.start_testing()

    print("Load testing completed")


@cli.command()
def status():
    documents_count = get_document_count()
    print(f"Server is ready. Document count: {documents_count}")


if __name__ == "__main__":
    cli()
