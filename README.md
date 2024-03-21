# KV-Sysyem

## Overview

This project includes a frontend service built with Actix-web and a backend service using Tonic gRPC. It's a simple key-value store with REST API endpoints exposed by the frontend service, which internally communicates with the backend gRPC service for data processing and storage.

## Prerequisites

Before you start, ensure you have the following installed on your system:

- **Docker**: [Download Docker](https://www.docker.com/get-started) and ensure it's running.
- **Docker Compose**: Comes installed with Docker Desktop for Windows and Mac. For Linux, you may need to [install it separately](https://docs.docker.com/compose/install/).

## Getting Started

### Clone the Repository

```bash
git clone https://github.com/dawid-majka/kv-system.git
cd cd kv-system
```

### Running services:

To start both services from root of a project:

```bash
docker-compose up --build
```

### Sending Requests:

Interact with the frontend service using `curl` or any HTTP client:

**POST Request Example**

```bash
curl -X POST http://localhost:8000/ \
     -H "Content-Type: application/json" \
     -d '{"key":"key1", "value":"value1"}'
```

**GET Request Example**

```bash
curl http://localhost:8000/key1
```

Additional request are inside requests.http file.

### You can also run services locally:

## Prerequisites

Ensure you have the following installed on your system:

- **Rust**: [Install Rust](https://www.rust-lang.org/tools/install)
- **Protobuf Compiler**: [Install protobuf compiler](https://grpc.io/docs/protoc-installation/)

**In root directory run:**

```bash
cargo run -p backend
cargo run -p frontend
```

### Tests

```
cargo test -p backend
cargo test -p frontend
```
