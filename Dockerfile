# Build stage
FROM rust:alpine as builder

# Add musl-dev and openssl for compiling dependencies
RUN apk add --no-cache musl-dev openssl-dev

# Create a new empty shell project
RUN USER=root cargo new --bin matrix_bot
WORKDIR /matrix_bot

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# This trick will cache our deps
RUN cargo build --release
RUN rm src/*.rs

# Now that the dependencies are built, copy your source code
COPY ./src ./src

# Build for release.
RUN rm ./target/release/deps/matrix_bot*
RUN cargo build --release

# Final stage
FROM alpine:latest

# Install libgcc (might be needed for some dependencies) and ca-certificates
RUN apk add --no-cache libgcc ca-certificates

# Copy the binary from the builder stage
COPY --from=builder /matrix_bot/target/release/matrix_bot /usr/local/bin/matrix_bot

# Set the working directory to /usr/local/bin
WORKDIR /usr/local/bin

# Set the entrypoint to your binary
ENTRYPOINT ["./matrix_bot"]
