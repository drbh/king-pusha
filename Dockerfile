# First stage: Build the application
FROM rust:1.68 as builder

WORKDIR /usr/src/app

# Copy your source code
COPY . .

# Build the application in release mode
RUN cargo build --release

# Second stage: Create the runtime image
FROM debian:buster-slim

# Add necessary libraries (you might need more, depending on your application)
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /root/

# Copy the binary from the build stage
COPY --from=builder /usr/src/app/target/release/king-pusha .

# Copy web assets
COPY --from=builder /usr/src/app/web ./web

# Expose the port your application uses (change the port if necessary)
EXPOSE 8081

# Specify the command to run your application
CMD ["./king-pusha"]
