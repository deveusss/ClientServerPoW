# Use the official Rust image as the base image
FROM rust as builder

# Create a new directory for our application code
WORKDIR /clientapp

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml Cargo.lock ./

# Copy the source code to the container
COPY src ./src

# Build the application
RUN cargo build --release

# Create a new stage with a smaller base image
FROM debian:buster-slim

# Install any necessary runtime dependencies
RUN apt-get update && apt-get install -y libssl-dev

WORKDIR /clientapp

# Copy the built application from the previous stage
COPY --from=builder /clientapp/target/release/client /

# Set the entrypoint for the container
CMD ["/client", "0.0.0.0:8080"]
