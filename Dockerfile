FROM rust:1.84-slim-bookworm AS builder

# Create a new empty shell project
RUN USER=root cargo new --bin reporteer
WORKDIR /app

RUN export DEBIAN_FRONTEND=noninteractive && \
  apt update && \
  apt install -y \
  apt-transport-https \
  autoconf \
  automake \
  autotools-dev \
  binutils \
  build-essential \
  ca-certificates \
  clang \
  curl \
  curl \
  file \
  gnupg \
  gnupg-agent \
  libssl-dev \
  libtool \
  llvm \
  lsb-release \
  lsb-release \
  make \
  net-tools \
  openssl \
  pkg-config \
  python3 \
  unzip \
  vim \
  wget \
  xutils-dev \
  xxd \
  && \
  rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

COPY templates /app/templates

## Cache dependencies
#RUN mkdir src && echo "fn main() {}" > src/main.rs && \
#    cargo build --release && \
#    rm -rf src

# Copy source code
COPY src ./src/

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install necessary runtime libraries
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary and supporting files
COPY --from=builder /app/target/release/reporteer /app/reporteer
COPY --from=builder /app/templates /app/templates

# Create a non-root user for the runtime stage
RUN adduser --disabled-password --gecos "" --home /nonexistent --shell /sbin/nologin --no-create-home --uid 10001 appuser

# Set the user to non-root
USER appuser

# Configure environment
ENV RUST_LOG=info
ENV REPORTEER_SERVER_PORT=3000
ENV REPORTEER_ENDPOINT_URL=http://127.0.0.1:8006/derived_key

# Expose the application port
EXPOSE 3000

# Run the binary
WORKDIR /app
CMD ["/app/reporteer"]
