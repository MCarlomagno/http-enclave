use tokio::{
    io,
    net::{TcpListener, TcpStream},
};
#[cfg(feature = "vsock")]
use vsock::VsockStream;

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
        tokio::spawn(async move {
            #[cfg(feature = "vsock")]
            {
                let mut enclave = VsockStream::connect((enclave_cid, enclave_port)).unwrap();
                let _ = io::copy_bidirectional(&mut client, &mut enclave).await;
            }
            #[cfg(not(feature = "vsock"))]
            {
                let mut enclave = TcpStream::connect("127.0.0.1:5005").await.unwrap();
                let _ = io::copy_bidirectional(&mut client, &mut enclave).await;
            }
        });
    }
}
