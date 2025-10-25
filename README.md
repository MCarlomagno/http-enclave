# HTTP Enclave

End-to-end encrypted communication where the host proxy cannot read traffic. TLS termination happens inside the enclave.

## What is this?

- **Host Proxy**: Forwards encrypted traffic to enclave (cannot decrypt)
- **Enclave**: HTTP server with TLS termination and encryption keys
- **Security**: Infrastructure operator cannot access decrypted data

## Build & Test

### Any Platform (TCP)
```bash
# Docker testing
make test-tcp

# Local testing
make build-local
make run-local
```

### Linux with vsock
```bash
# Docker testing
make test-vsock

# Local testing  
make build-local-vsock
make run-local-vsock
```

## Test the System

```bash
# Send encrypted data
curl -k -X POST https://localhost/private-data \
  -d '{"secret":"my-data"}' \
  -H 'content-type: application/json'

# Response: {"request_id":"...","status":"stored"}
```

## Configuration

### Environment Variables

**Enclave:**
- `USE_TLS=true` - Enable TLS
- `ENCLAVE_PORT=8443` - TCP port (or 5005 for vsock)
- `ENCLAVE_KEY_BASE64=...` - Encryption key

**Host:**
- `BIND_ADDR=0.0.0.0:443` - Listen address
- `ENCLAVE_ADDR=enclave:8443` - Enclave address (TCP)
- `ENCLAVE_CID=2` - Enclave CID (vsock)

### Feature Flags

- `tcp` (default) - Use TCP communication
- `vsock` - Use vsock communication (Linux only)

## Commands

```bash
make help              # Show all commands
make test-tcp          # Test with Docker (TCP)
make test-vsock        # Test with Docker (vsock)
make build-local       # Build for local development
make clean             # Clean build artifacts
```