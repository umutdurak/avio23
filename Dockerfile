# Use an official Rust runtime as a parent image
FROM rust:1.88

WORKDIR /usr/src/avio23

# Install build dependencies required by a653rs-linux procfs and musl target
RUN apt-get update && \
    apt-get install -y cmake build-essential musl-tools

RUN rustup target add $(uname -m)-unknown-linux-musl

# Copy the entire workspace (Platform and Implementation)
COPY . .

# Build the partitions with the musl target (static linking)
WORKDIR /usr/src/avio23/implementation
RUN cargo build --release --target $(uname -m)-unknown-linux-musl

# Build the hypervisor natively
WORKDIR /usr/src/avio23/platform/a653rs-linux
RUN cargo build --release --package a653rs-linux-hypervisor && \
    cp target/release/a653rs-linux-hypervisor /usr/local/bin/a653rs-linux-hypervisor

# To run the hypervisor, we set the PATH so it can discover the partition binaries
ENV PATH="/usr/src/avio23/implementation/target/aarch64-unknown-linux-musl/release:/usr/src/avio23/implementation/target/x86_64-unknown-linux-musl/release:${PATH}"

# We switch to the hypervisor directory as the execution root
WORKDIR /usr/src/avio23/platform/a653rs-linux

