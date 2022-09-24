use anyhow::{anyhow, Error};
use ethers::{
    prelude::*,
    utils::rlp::{Decodable, Rlp},
};
use futures::{sink::SinkExt, stream::StreamExt};
use nym_websocket::responses::ServerResponse;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

pub const DEFAULT_NYM_CLIENT_ENDPOINT: &str = "ws://localhost:1977";

/// Server maintains a connection to a Nym client and upon receiving an Ethereum
/// transaction, it submits to an Ethereum node.
pub struct Server<M: Middleware + 'static> {
    ws: WebSocketStream<TcpStream>,
    m: M,
}

impl<M: Middleware + 'static> Server<M> {
    pub async fn new(endpoint: Option<String>, m: M) -> Result<Self, Error> {
        let (ws, _) =
            connect_async(endpoint.unwrap_or(DEFAULT_NYM_CLIENT_ENDPOINT.to_string())).await?;

        Ok(Server { ws, m })
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

            let transaction_res = decode_transaction(&msg_bytes.message);
            if transaction_res.is_err() {
                warn!("{:?}", transaction_res.err());
                continue;
            }

            let receipt_res = self.submit_transaction(&transaction_res.unwrap()).await;
            if receipt_res.is_err() {
                warn!("{:?}", receipt_res.err());
                continue;
            }

            info!("transaction included: {:?}", receipt_res.unwrap());
        }
    }

    pub async fn submit_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<TransactionReceipt, Error> {
        let pending_tx = self.m.send_transaction(transaction, None).await?;
        info!("submitted transaction: hash {:?}", pending_tx.tx_hash());
        let maybe_receipt = pending_tx.await?;
        if maybe_receipt.is_none() {
            return Err(anyhow!("did not receive transaction receipt"));
        }
        Ok(maybe_receipt.unwrap())
    }
}

fn decode_transaction(bytes: &[u8]) -> Result<Transaction, Error> {
    let rlp = Rlp::new(bytes);
    Transaction::decode(&rlp)
        .map_err(|e| anyhow!("failed to decode transaction from message: {:?}", e))
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

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}
