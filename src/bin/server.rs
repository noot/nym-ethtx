use anyhow;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, WebSocketStream};

pub struct Server {
    ws: WebSocketStream<TcpStream>,
}

impl Server {
    async fn new(endpoint: Option<String>) -> Result<Self, anyhow::Error> {
        let (mut ws, _) =
            connect_async(endpoint.unwrap_or("ws://localhost:1977".to_string())).await?;

        Ok(Server { ws })
    }
}

fn main() {}
