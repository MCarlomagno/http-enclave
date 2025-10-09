# HTTP TEE

End to end encrypted communication between clients and an isolated enclave. The infrastructure operator cannot read the traffic.

## How it works

### Components

Host proxy (crates/host):
- Listens for TCP connections on port 443
- Forwards all traffic to the enclave
- Cannot decrypt TLS (no access to private keys)

Enclave (crates/enclave):
- HTTP API server with TLS termination
- Holds encryption keys in memory
- Processes sensitive operations

### Traffic flow

```
Client -> HTTPS -> Host Proxy (encrypted passthrough) -> Enclave (TLS termination)
```

The host proxy only sees encrypted bytes. TLS termination happens inside the enclave.

### Security

What the operator cannot access:
- TLS private key
- Encryption keys
- HTTP request/response content
- Decrypted data

What the operator can access:
- Encrypted traffic patterns
- Connection metadata
- Ciphertext at rest

## Local setup

Generate enclave key:
```bash
export ENCLAVE_KEY_BASE64=$(openssl rand -base64 32)
```

Generate TLS certificate:
```bash
openssl req -x509 -newkey rsa:2048 -keyout key.pem -out cert.pem -days 365 -nodes -subj "/CN=localhost"
```

Setup environment variables:

Copy the contents of env.example into .env and update the values.
```bash
cp env.example .env
```

Run enclave:
```bash
cargo run --bin enclave
```

Run host proxy:
```bash
cargo run --bin host
```

Test:
```bash
curl -k -XPOST https://localhost:443/private-data -d '{"hello":"world"}' -H 'content-type: application/json'
curl -k https://localhost:443/private-data
```

## Production deployment

Build with vsock support:
```bash
cargo build -p enclave --features vsock --release
cargo build -p host --features vsock --release
```

Enclave configuration:
- Set ENCLAVE_PORT (e.g., 5005)
- Load TLS certificate and key securely
- Use KMS with attestation for ENCLAVE_KEY_BASE64

Host configuration:
- Set ENCLAVE_CID (e.g., 16)
- Set ENCLAVE_PORT
- Run on EC2 parent instance

## Configuration

Enclave environment variables:
- USE_TLS: Enable TLS (default: true)
- TLS_CERT_PATH: Certificate file path (default: cert.pem)
- TLS_KEY_PATH: Private key file path (default: key.pem)
- ENCLAVE_KEY_BASE64: Base64-encoded 32-byte AES key

Host environment variables:
- BIND_ADDR: Listen address (default: 0.0.0.0:443)
- ENCLAVE_CID: VSOCK CID for production
- ENCLAVE_PORT: Enclave port

## Endpoints

POST /private-data: Store encrypted data
GET /private-data: Retrieve decrypted data

## Limitations

- Encryption/decryption endpoints are skeleton implementations
- Keys loaded from environment (development only)
- No KMS integration
- No attestation verification
- Operator can observe traffic patterns and sizes
