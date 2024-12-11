import os
import time
import threading
from send import get_document_count, upload_file


class LoadTester:
    num_threads: int
    max_dataset_size: int
    files: list

    def __init__(self, num_threads: int, data_dir: str, max_dataset_size: int):
        self.num_threads = num_threads
        self.max_dataset_size = max_dataset_size
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
        print(f"Створення потоків\t\t{end - start} секунд")
        start = end

        for thread in threads:
            thread.start()

        for thread in threads:
            thread.join()

        end = time.time()
        print(f"Завантаження документів:\t{end - start} секунд")

        desired_document_count = initial_document_count + len(self.files)

        while get_document_count() < desired_document_count:
            pass

        end = time.time()
        print(f"Оновлення індекса:\t\t{end - start} секунд")

    def load_data(self, data_dir):
        self.files = [
            os.path.abspath(os.path.join(data_dir, f))
            for f in os.listdir(data_dir)[:self.max_dataset_size]
        ]

    def upload_chunk(self, chunk):
        for file_path in chunk:
            response = upload_file(file_path)

            if response != "SUCCESS":
                print(f"Failed to upload file '{file_path}'.")
