# Reporteer

A secure service for interacting with TEE platforms inside confidential containers, running in Kata containers on AMD SEV SNP and Intel TDX.

## Features

- Fetches and hashes derived keys from TEE platform
- Exposes hash via web interface and JSON API
- Runs in confidential containers
- Supports AMD SEV SNP and Intel TDX
- Follows security best practices and 12-factor app principles

## Prerequisites

- Rust 1.73 or later
- Docker
- Kubernetes cluster with Kata Containers support
- AMD SEV SNP or Intel TDX capable hardware

## Quick Start

1. Build the application:
```bash
cargo build --release
```

2. Build the Docker image:
```bash
docker build -t reporteer .
```

3. Deploy to Kubernetes:
```bash
kubectl apply -f k8s/deployment.yaml
```

## Configuration

Configuration is done via environment variables:

- `REPORTEER_SERVER_PORT`: Web server port (default: 3000)
- `REPORTEER_ENDPOINT_URL`: TEE platform endpoint (default: http://127.0.0.1:8006/derived_key)
- `REPORTEER_LOG_LEVEL`: Logging level (default: info)

See `.env.example` for all available options.

## API Endpoints

- `GET /`: HTML page showing the derived key hash
- `GET /api/hash`: JSON endpoint returning the hash
- `GET /health`: Health check endpoint

## Development

1. Clone the repository:
```bash
git clone https://github.com/yourusername/reporteer.git
```

2. Copy environment file:
```bash
cp .env.example .env
```

3. Run tests:
```bash
cargo test
```

4. Run locally:
```bash
cargo run
```

## Security

- Runs as non-root user
- Uses confidential computing features
- Implements secure coding practices
- Regular security updates

## Docker Container

The application runs in a minimal Debian-based container with:
- Non-root user
- Minimal dependencies
- Security hardening
- Multi-stage build

## Kubernetes Deployment

Includes:
- Kata Containers runtime
- Resource limits
- Health checks
- ConfigMap for configuration
- Security context

## License

MIT License - see LICENSE file for details

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Requestwift TEE measurement Reporter
