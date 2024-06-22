# Dockerfile for server
FROM rust:1.70

WORKDIR /usr/src/app

COPY . .

RUN cargo install --path .

CMD ["./target/release/your_server_binary"]
