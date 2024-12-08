# Course Work Parallel Computing


## How to

### Server
```bash
$ export RUST_LOG=debug
$ cargo run
```


### Python Client
```
$ cd clients/python

$ # Installation
$ python3 -m venv .venv
$ source .venv/bin/acitvate
$ pip3 install -r requirements.txt

$ # Usage
$ python3 main.py upload --file-path file.txt
$ python3 main.py search --term query
$ python3 main.py download --document-id 4
$ python3 main.py delete --document-id 4
```


### Rust Client
```
cd client/rust

# Installation
cargo build

# Usage
cargo run -- download --document-id 4
cargo run -- search --term driven
cargo run -- upload le
```


### Load Testing
Only Python client supports load testing.

```
$ cd clients/python

$ # Installation
$ python3 -m venv .venv
$ source .venv/bin/acitvate
$ pip3 install -r requirements.txt

$ # Usage
$ python3 main.py load --num-threads=10 --data-dir /path/to/documents
```
