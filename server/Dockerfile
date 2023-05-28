# Use the official Rust image as the base image
FROM rust as builder

# Create a new directory for our application code
WORKDIR /serverapp

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

WORKDIR /serverapp

# Copy the built application from the previous stage
COPY --from=builder /serverapp/target/release/server /

# Set the entrypoint for the container
CMD ["/server", "--bind-address", "0.0.0.0:8080"]
