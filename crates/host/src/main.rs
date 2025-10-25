use tokio::{
    io,
    net::{TcpListener, TcpStream},
};

#[cfg(feature = "vsock")]
use tokio_vsock::{VsockAddr, VsockStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listen = std::env::var("BIND_ADDR").unwrap_or("0.0.0.0:443".into());

    #[cfg(feature = "vsock")]
    let enclave_cid: u32 = std::env::var("ENCLAVE_CID")?.parse()?;
    #[cfg(feature = "vsock")]
    let enclave_port: u32 = std::env::var("ENCLAVE_PORT")?.parse()?;

    let ln = TcpListener::bind(&listen).await?;
    println!("host proxy on {}", listen);

    loop {
        let (mut client, _) = ln.accept().await?;
        println!("[HOST] New client connection accepted");
        tokio::spawn(async move {
            #[cfg(feature = "vsock")]
            {
                let addr = VsockAddr::new(enclave_cid, enclave_port);
                let mut enclave = VsockStream::connect(addr).await.unwrap();
                let _ = io::copy_bidirectional(&mut client, &mut enclave).await;
            }
            #[cfg(not(feature = "vsock"))]
            {
                let addr = std::env::var("ENCLAVE_ADDR").unwrap_or_else(|_| "127.0.0.1:8443".into());
                println!("[HOST] Connecting to enclave at: {}", addr);
                let mut enclave = TcpStream::connect(addr).await.unwrap();
                println!("[HOST] Connected to enclave, forwarding encrypted traffic...");
                let _ = io::copy_bidirectional(&mut client, &mut enclave).await;
                println!("[HOST] Connection closed");
            }
        });
    }
}
