use tokio::{
    io::AsyncWriteExt,
    net::{TcpSocket, TcpStream},
};

use crate::error::ServerError;

pub struct Client {
    socket: TcpStream,
}

impl Client {
    pub async fn init() -> Result<Self, ServerError> {
        let socket = CreateSocket().await?;

        Ok(Client {
            socket,
        })
    }

    pub async fn send(
        &mut self,
        data: Vec<u8>,
    ) -> Result<(), ServerError> {

        self.socket.write_all(&data).await?;

        Ok(())
    }
}

async fn CreateSocket() -> Result<TcpStream, ServerError> {

    let addr = std::env::var("docker_addr")?;

    let socket = TcpSocket::new_v4()?;

    let stream =
        socket.connect(addr.parse().unwrap()).await?;

    Ok(stream)
}