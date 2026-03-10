# Use an official Rust runtime as a parent image
FROM rust:1.80

WORKDIR /usr/src/avio23

# Install musl-tools and build dependencies required by a653rs-linux procfs
RUN apt-get update && \
    apt-get install -y musl-tools cmake build-essential && \
    rustup target add x86_64-unknown-linux-musl

# Copy the entire workspace (Platform and Implementation)
COPY . .

# Build the partitions (This caches the dependencies and compiles our CPMs)
WORKDIR /usr/src/avio23/implementation
RUN cargo build --release --target x86_64-unknown-linux-musl

# To run the hypervisor, we set the PATH so it can discover the partition binaries
ENV PATH="/usr/src/avio23/implementation/target/x86_64-unknown-linux-musl/release:${PATH}"

# We switch to the hypervisor directory as the execution root
WORKDIR /usr/src/avio23/platform/a653rs-linux
