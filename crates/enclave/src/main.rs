use anyhow::Result;
use axum::{routing::post, body::Bytes, http::StatusCode, Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use base64::Engine;
use chacha20poly1305::{aead::{Aead, KeyInit}, AeadCore, XChaCha20Poly1305};
use rand_core::OsRng;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct StoreMsg {
    request_id: String,
    content_type: String,
    nonce_b64: String,
    ciphertext_b64: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Generate a self-signed cert at boot (key never leaves enclave).
    // In prod, swap to NGINX + ACM for Enclaves.
    let (cert_pem, key_pem) = self_signed_cert_pem()?;
    let tls = RustlsConfig::from_pem(cert_pem.into_bytes(), key_pem.into_bytes()).await?;

    // One simple endpoint
    let app = Router::new().route("/private-data", post(private_data));

    // Listen on 0.0.0.0:8443 (host proxy will bridge host:443 → enclave:8443)
    println!("[ENCLAVE] Starting TLS server on 0.0.0.0:8443");
    axum_server::bind_rustls(([0, 0, 0, 0], 8443).into(), tls)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn private_data(body: Bytes) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("[ENCLAVE] Received private data request");
    println!("[ENCLAVE] Request body (decrypted): {}", String::from_utf8_lossy(&body));
    
    // Encrypt inside the enclave
    let key = XChaCha20Poly1305::generate_key(&mut OsRng);
    let cipher = XChaCha20Poly1305::new(&key);
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ct = cipher.encrypt(&nonce, body.as_ref()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    println!("[ENCLAVE] Data encrypted with key length: {} bytes", key.len());

    let request_id = uuid_like();
    let msg = StoreMsg {
        request_id: request_id.clone(),
        content_type: "application/octet-stream".into(),
        nonce_b64: base64::engine::general_purpose::STANDARD.encode(nonce),
        ciphertext_b64: base64::engine::general_purpose::STANDARD.encode(ct),
    };

    // Fire-and-forget to host storage over vsock (parent CID=3)
    if let Err(e) = send_to_host(&msg).await {
        eprintln!("send_to_host error: {e:?}");
        // you can decide to fail the request or keep going
    }

    Ok(Json(serde_json::json!({
        "request_id": request_id,
        "status": "stored"
    })))
}

// ---- enclave → host storage over vsock (parent CID is always 3) ----

const HOST_CID: u32 = 3;
const HOST_STORE_PORT: u32 = 7001;

async fn send_to_host(msg: &StoreMsg) -> Result<()> {
    #[cfg(feature = "vsock")]
    {
        use tokio::io::AsyncWriteExt;
        use tokio_vsock::VsockStream;

        let mut s = VsockStream::connect(tokio_vsock::VsockAddr::new(HOST_CID, HOST_STORE_PORT)).await?;
        let bytes = serde_json::to_vec(msg)?;
        s.write_all(&bytes).await?;
    }
    
    #[cfg(feature = "tcp")]
    {
        // For TCP mode, we'll just log that we would send to host
        println!("[ENCLAVE] Would send to host over TCP: {:?}", msg);
    }
    
    Ok(())
}

// ---- util: self-signed cert at boot (demo only) ----
fn self_signed_cert_pem() -> Result<(String, String)> {
    let cert = rcgen::generate_simple_self_signed(vec!["enclave.local".into()])?;
    let cert_pem = cert.cert.pem();
    let key_pem = cert.signing_key.serialize_pem();
    Ok((cert_pem, key_pem))
}

// simple unique id; replace with real UUID if desired
fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{t:x}")
}