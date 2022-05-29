# cheese-grader/runner

## Setup
Install [Rust](https://rustup.rs), [just](https://github.com/casey/just#installation), and [Docker](https://get.docker.com/).

Build the Docker images
```sh
just build-all
# or:
just build python,java
```

Run some code
```sh
# Usage: ./runner IMAGE DIRECTORY ENTRYPOINT
cargo run cheese-grader/runner-java:latest test/java Main.java
```