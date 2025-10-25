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

## Deployment (AWS Nitro Enclaves)

- Prereqs: EC2 Nitro instance (e.g., c6i), Nitro Enclaves enabled, Docker + `nitro-cli` installed, inbound 443 allowed.

1) Build EIF from the enclave image (on the parent instance)

```bash
docker build -f Dockerfile.enclave.vsock -t http-enclave-enclave .
sudo nitro-cli build-enclave --docker-uri http-enclave-enclave:latest --output-file enclave.eif
```

2) Run the enclave and get its CID

```bash
sudo nitro-cli run-enclave --eif-path enclave.eif --cpu-count 2 --memory 1024
CID=$(sudo nitro-cli describe-enclaves | grep -m1 EnclaveCID | awk '{print $2}')
echo "Enclave CID: $CID"
```

3) Run the host proxy on the parent with vsock

```bash
cargo build -p host --features vsock --release
BIND_ADDR=0.0.0.0:443 ENCLAVE_CID=$CID ENCLAVE_PORT=5005 ./target/release/host
```

4) Test from your workstation

```bash
curl -k -X POST https://<parent-ec2-public-ip>/private-data \
  -d '{"secret":"my-data"}' \
  -H 'content-type: application/json'
```

Notes:
- In Nitro, the enclave reaches the parent at CID 3; the parent connects to the enclave using the CID from `describe-enclaves`.
- Nitro Enclaves have no networking; enclave↔parent communication is via vsock only.