.PHONY: help build-enclave build-host build-eif run-enclave stop-enclave describe-enclave console clean

help:
	@echo "AWS Nitro Enclave Deployment Commands"
	@echo ""
	@echo "Local Development:"
	@echo "  make build-local         Build both host and enclave for local dev (no vsock)"
	@echo "  make run-local           Run locally with cargo"
	@echo ""
	@echo "Production (AWS Nitro):"
	@echo "  make build-enclave       Build enclave Docker image"
	@echo "  make build-host          Build host Docker image"
	@echo "  make build-eif           Convert enclave Docker image to EIF file"
	@echo "  make run-enclave         Launch the enclave on EC2"
	@echo "  make describe-enclave    Show running enclave status"
	@echo "  make console             Attach to enclave console"
	@echo "  make stop-enclave        Terminate the enclave"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean               Remove build artifacts"

# Local development (no vsock)
build-local:
	cargo build -p enclave
	cargo build -p host

run-local:
	@echo "Run in separate terminals:"
	@echo "  Terminal 1: cargo run --bin enclave"
	@echo "  Terminal 2: cargo run --bin host"

# Production builds
build-enclave:
	docker build -f Dockerfile.enclave -t enclave:latest .

build-host:
	docker build -f Dockerfile.host -t host:latest .

build-eif: build-enclave
	nitro-cli build-enclave \
		--docker-uri enclave:latest \
		--output-file enclave.eif

# Enclave operations (run on EC2 parent instance)
run-enclave:
	nitro-cli run-enclave \
		--cpu-count 2 \
		--memory 512 \
		--enclave-cid 16 \
		--eif-path enclave.eif \
		--debug-mode

describe-enclave:
	nitro-cli describe-enclaves

console:
	@ENCLAVE_ID=$$(nitro-cli describe-enclaves | grep -oP '"EnclaveID": "\K[^"]+' | head -1); \
	if [ -z "$$ENCLAVE_ID" ]; then \
		echo "No running enclave found"; \
	else \
		nitro-cli console --enclave-id $$ENCLAVE_ID; \
	fi

stop-enclave:
	@ENCLAVE_ID=$$(nitro-cli describe-enclaves | grep -oP '"EnclaveID": "\K[^"]+' | head -1); \
	if [ -z "$$ENCLAVE_ID" ]; then \
		echo "No running enclave found"; \
	else \
		nitro-cli terminate-enclave --enclave-id $$ENCLAVE_ID; \
	fi

clean:
	cargo clean
	rm -f enclave.eif
	docker rmi enclave:latest host:latest 2>/dev/null || true
