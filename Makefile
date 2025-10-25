.PHONY: help build-local run-local test-tcp test-vsock build-tcp build-vsock clean

help:
	@echo "HTTP Enclave Testing Commands"
	@echo ""
	@echo "Local Development:"
	@echo "  make build-local         Build both host and enclave for local dev (TCP mode)"
	@echo "  make run-local           Run locally with cargo (TCP mode)"
	@echo ""
	@echo "Docker Testing:"
	@echo "  make test-tcp            Test with Docker using TCP communication"
	@echo "  make test-vsock          Test with Docker using vsock communication (Linux only)"
	@echo "  make build-tcp           Build Docker images for TCP testing"
	@echo "  make build-vsock         Build Docker images for vsock testing"
	@echo ""
	@echo "Direct Testing (Linux with vsock):"
	@echo "  make run-local-vsock     Run locally with vsock (Linux only)"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean               Remove build artifacts and Docker images"

# Local development (TCP mode)
build-local:
	cargo build -p enclave --features tcp
	cargo build -p host --features tcp

run-local:
	@echo "Run in separate terminals:"
	@echo "  Terminal 1: cargo run --bin enclave --features tcp"
	@echo "  Terminal 2: cargo run --bin host --features tcp"

# Local development (vsock mode - Linux only)
build-local-vsock:
	cargo build -p enclave --features vsock
	cargo build -p host --features vsock

run-local-vsock:
	@echo "Run in separate terminals (Linux with vsock support):"
	@echo "  Terminal 1: ENCLAVE_PORT=5005 cargo run --bin enclave --features vsock"
	@echo "  Terminal 2: BIND_ADDR=0.0.0.0:443 ENCLAVE_CID=2 ENCLAVE_PORT=5005 cargo run --bin host --features vsock"

# Docker TCP testing
build-tcp:
	docker build -f Dockerfile.enclave.tcp -t enclave-tcp .
	docker build -f Dockerfile.host.tcp -t host-tcp .

test-tcp: build-tcp
	docker compose -f docker-compose.tcp.yml up --build

# Docker vsock testing (Linux only)
build-vsock:
	docker build -f Dockerfile.enclave.vsock -t enclave-vsock .
	docker build -f Dockerfile.host.vsock -t host-vsock .

test-vsock: build-vsock
	docker compose -f docker-compose.vsock.yml up --build

clean:
	cargo clean
	docker compose -f docker-compose.tcp.yml down 2>/dev/null || true
	docker compose -f docker-compose.vsock.yml down 2>/dev/null || true
	docker rmi enclave-tcp host-tcp enclave-vsock host-vsock 2>/dev/null || true
