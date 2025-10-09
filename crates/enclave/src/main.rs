use aes_gcm::{Aes256Gcm, KeyInit};
use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use base64::prelude::*;
use tokio::net::TcpListener;

#[derive(Clone)]
struct EnclaveState {
    _cipher: Aes256Gcm,
}

fn load_enclave_state(enclave_key_base64: &str) -> anyhow::Result<EnclaveState> {
    let key_bytes = base64::prelude::BASE64_STANDARD.decode(enclave_key_base64)?;
    if key_bytes.len() != 32 {
        anyhow::bail!(
            "ENCLAVE_KEY_BASE64 must decode to exactly 32 bytes, got {}",
            key_bytes.len()
        );
    }
    Ok(EnclaveState {
        _cipher: Aes256Gcm::new(key_bytes.as_slice().into()),
    })
}

async fn load_tls_config() -> anyhow::Result<axum_server::tls_rustls::RustlsConfig> {
    // Load TLS certificate and key from files or environment
    let cert_path = std::env::var("TLS_CERT_PATH").unwrap_or_else(|_| "cert.pem".into());
    let key_path = std::env::var("TLS_KEY_PATH").unwrap_or_else(|_| "key.pem".into());

    let config =
        axum_server::tls_rustls::RustlsConfig::from_pem_file(&cert_path, &key_path).await?;

    Ok(config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let enclave_key_base64 = std::env::var("ENCLAVE_KEY_BASE64").unwrap();
    let state = load_enclave_state(&enclave_key_base64)?; // holds AES/KMS client, etc.
    let app = Router::new()
        .route("/private-data", post(post_private))
        .route("/private-data", get(get_private))
        .with_state(state);

    let addr = "0.0.0.0:5005";

    let use_tls = std::env::var("USE_TLS").unwrap_or_else(|_| "true".into()) == "true";

    if use_tls {
        println!("Starting enclave with TLS on {}", addr);
        let tls_config = load_tls_config().await?;
        axum_server::bind_rustls(addr.parse()?, tls_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        println!(
            "Starting enclave without TLS on {} (WARNING: unencrypted)",
            addr
        );
        axum::serve(TcpListener::bind(&addr).await?, app.into_make_service()).await?;
    }

    Ok(())
}
async fn post_private(State(_st): State<EnclaveState>, _body: Bytes) -> (StatusCode, String) {
    (StatusCode::CREATED, "stored".into())
}
async fn get_private(State(_st): State<EnclaveState>) -> (StatusCode, Vec<u8>) {
    (StatusCode::OK, b"plaintext".to_vec())
}
