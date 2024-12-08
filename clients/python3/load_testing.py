import os
import time
import threading
from send import get_document_count, upload_file

MAX_DATASET_SIZE = 10000


class LoadTester:
    num_threads: int
    files: list

    def __init__(self, num_threads: int, data_dir: str):
        self.num_threads = num_threads
        self.load_data(data_dir)
        print("Files loaded:", len(self.files))

    def start_testing(self):
        initial_document_count = get_document_count()

        chunk_size = len(self.files) // self.num_threads

        start = time.time()

        threads = []
        for i in range(0, len(self.files), chunk_size):
            chunk = self.files[i : i + chunk_size]
            thread = threading.Thread(target=self.upload_chunk, args=(chunk,))
            threads.append(thread)

        end = time.time()
        print(f"Threads:\t\t{end - start} seconds")
        start = end

        for thread in threads:
            thread.start()

        for thread in threads:
            thread.join()

        end = time.time()
        print(f"Upload:\t\t\t{end - start} seconds")

        desired_document_count = initial_document_count + len(self.files)

        while get_document_count() < desired_document_count:
            pass

        end = time.time()
        print(f"Index updates:\t\t{end - start} seconds")

    def load_data(self, data_dir):
        self.files = [
            os.path.abspath(os.path.join(data_dir, f))
            for f in os.listdir(data_dir)[:MAX_DATASET_SIZE]
        ]

    def upload_chunk(self, chunk):
        for file_path in chunk:
            response = upload_file(file_path)

            if response != "SUCCESS":
                print(f"Failed to upload file '{file_path}'.")
