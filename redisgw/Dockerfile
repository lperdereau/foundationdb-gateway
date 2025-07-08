FROM rust:1.85-bullseye as builder

WORKDIR /usr/src/redis
RUN apt-get update \
    && apt-get install -y clang libclang-dev llvm-dev pkg-config \
    && wget -O /tmp/fdb-client.deb https://github.com/apple/foundationdb/releases/download/7.3.63/foundationdb-clients_7.3.63-1_amd64.deb \
    && dpkg -i /tmp/fdb-client.deb

COPY . .

RUN cargo build --release

# ---- Runtime Stage ----
FROM debian:bookworm-slim

# Install any runtime dependencies here (e.g., libssl for FoundationDB if needed)
RUN apt-get update \
    && apt-get install -y libssl-dev ca-certificates wget \
    && wget -O /tmp/fdb-client.deb https://github.com/apple/foundationdb/releases/download/7.3.63/foundationdb-clients_7.3.63-1_amd64.deb \
    && dpkg -i /tmp/fdb-client.deb \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/redis/target/release/redisgw /app/redisgw

# Set the default command to run your binary
CMD ["./redisgw"]
