use anyhow::{anyhow, Error};
use ethers::prelude::*;
use futures::{sink::SinkExt, stream::StreamExt};
use nym_websocket::responses::ServerResponse;
use structopt::StructOpt;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

use nym_ethtx::{Network, DEFAULT_NYM_CLIENT_ENDPOINT};

#[derive(StructOpt)]
struct Options {
    /// Nym websocket client endpoint. Default: ws://localhost:1977
    #[structopt(short, long, default_value = DEFAULT_NYM_CLIENT_ENDPOINT)]
    endpoint: String,

    /// Ethereum network to use.
    /// One of mainnet, goerli, or development.
    #[structopt(short, long, default_value = "development")]
    network: String,
}

/// Server maintains a connection to a Nym client and upon receiving an Ethereum
/// transaction, it submits to an Ethereum node.
pub struct Server {
    ws: WebSocketStream<TcpStream>,
    provider: Provider<Http>,
}

impl Server {
    pub async fn new(endpoint: String, provider: Provider<Http>) -> Result<Self, Error> {
        let (ws, _) = connect_async(endpoint).await?;
        Ok(Server { ws, provider })
    }

    pub async fn send_address_request(&mut self) -> Result<(), Error> {
        let req = nym_websocket::requests::ClientRequest::SelfAddress;
        let message = Message::Binary(req.serialize());
        self.ws.send(message).await?;
        Ok(())
    }

    pub async fn listen(&mut self) {
        while let Some(Ok(msg)) = self.ws.next().await {
            let res = parse_nym_message(msg);
            if res.is_err() {
                warn!("received unknown message: error {:?}", res.err());
                continue;
            }

            let msg_bytes = match res.unwrap() {
                ServerResponse::Received(msg_bytes) => {
                    debug!("received request {:?}", msg_bytes);
                    msg_bytes
                }
                ServerResponse::SelfAddress(addr) => {
                    info!("listening on {}", addr);
                    continue;
                }
                ServerResponse::Error(err) => {
                    error!("received error: {}", err);
                    continue;
                }
            };

            let receipt_res = self
                .submit_transaction(Bytes::from(msg_bytes.message))
                .await;
            if receipt_res.is_err() {
                warn!("{:?}", receipt_res.err());
                continue;
            }

            info!("transaction included: {:?}", receipt_res.unwrap());
        }
    }

    async fn submit_transaction(&self, transaction: Bytes) -> Result<TransactionReceipt, Error> {
        let pending_tx = self.provider.send_raw_transaction(transaction).await?;
        info!("submitted transaction: hash {:?}", pending_tx.tx_hash());
        let maybe_receipt = pending_tx.await?;
        if maybe_receipt.is_none() {
            return Err(anyhow!("did not receive transaction receipt"));
        }
        Ok(maybe_receipt.unwrap())
    }

    pub async fn close(&mut self) -> Result<(), Error> {
        self.ws
            .close(None)
            .await
            .map_err(|e| anyhow!("failed to close: {:?}", e))
    }
}

fn parse_nym_message(msg: Message) -> Result<ServerResponse, Error> {
    match msg {
        Message::Text(str) => ServerResponse::deserialize(&str.into_bytes())
            .map_err(|e| anyhow!("failed to deserialize text message: {:?}", e)),
        Message::Binary(bytes) => ServerResponse::deserialize(&bytes)
            .map_err(|e| anyhow!("failed to deserialize binary message: {:?}", e)),
        _ => Err(anyhow!("unknown message")),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .init();

    let options: Options = Options::from_args();

    let eth_endpoint = Network::from_str(&options.network).unwrap().get_endpoint();

    let provider =
        Provider::<Http>::try_from(eth_endpoint).expect("could not instantiate HTTP Provider");

    let mut server = Server::new(options.endpoint, provider).await.unwrap();
    server.send_address_request().await.unwrap();
    server.listen().await;
}

#[tokio::test]
async fn test_server() {
    use ethers::utils::Anvil;
    use nym_ethtx::DEFAULT_NYM_CLIENT_ENDPOINT;

    let anvil = Anvil::new().spawn();
    let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

    let mut server = Server::new(DEFAULT_NYM_CLIENT_ENDPOINT.to_string(), provider)
        .await
        .unwrap();
    server.send_address_request().await.unwrap();
    tokio::spawn(async move {
        server.listen().await;
    });
}
