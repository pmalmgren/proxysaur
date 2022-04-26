use anyhow::Result;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

pub async fn tunnel<T: AsyncRead + AsyncWrite>(
    client_socket: T,
    upstream_addr: &str,
) -> Result<()> {
    let upstream = TcpStream::connect(upstream_addr).await?;
    let (mut server_rh, mut server_wh) = tokio::io::split(upstream);
    let (mut client_rh, mut client_wh) = tokio::io::split(client_socket);

    tokio::join! {
        async {
            loop {
                let mut buf: Vec<u8> = vec![0; 2056];
                let bytes_read = match server_rh.read(&mut buf).await {
                    Ok(n_bytes) => n_bytes,
                    Err(error) => {
                        tracing::error!(%error, "Error reading bytes from server");
                        break;
                    },
                };
                if bytes_read == 0 {
                    tracing::debug!("Detected EOF from server.");
                    break;
                }
                match client_wh.write_all(&buf[0..bytes_read]).await {
                    Ok(_) => {},
                    Err(error) => {
                        tracing::error!(%error, "Error writing bytes to client.");
                        break;
                    }
                };
            }
        },
        async {
            loop {
                let mut buf: Vec<u8> = vec![0; 2056];
                let bytes_read = match client_rh.read(&mut buf).await {
                    Ok(n_bytes) => n_bytes,
                    Err(error) => {
                        tracing::error!(%error, "Error reading bytes from client.");
                        break;
                    },
                };
                if bytes_read == 0 {
                    tracing::debug!("Detected EOF from client.");
                    break;
                }
                match server_wh.write_all(&buf).await {
                    Ok(_) => {},
                    Err(error) => {
                        tracing::error!(%error, "Error writing bytes to server.");
                        break;
                    }
                };
            }
        }
    };

    Ok(())
}
