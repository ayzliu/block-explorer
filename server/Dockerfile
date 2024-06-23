FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the entire project into the container
COPY . .

# Build the project in release mode
RUN cargo build --release

# Create a new stage for the runtime image
FROM debian:buster-slim

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/your_server_binary /usr/local/bin/your_server_binary

# Expose the necessary port
EXPOSE 8000

# Set the entrypoint to the compiled binary
ENTRYPOINT ["your_server_binary"]
