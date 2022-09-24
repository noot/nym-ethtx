use anyhow::{anyhow, Error};
use ethers::prelude::*;
use futures::{sink::SinkExt, stream::StreamExt};
use nym_websocket::responses::ServerResponse;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::Network;

/// Server maintains a connection to a Nym client and upon receiving an Ethereum
/// transaction, it submits to an Ethereum node.
pub struct Server {
    ws: WebSocketStream<TcpStream>,
}

impl Server {
    pub async fn new(endpoint: String) -> Result<Self, Error> {
        let (ws, _) = connect_async(endpoint).await?;
        Ok(Server { ws })
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

            let data_res = parse_message_data(&msg_bytes.message);
            if data_res.is_err() {
                warn!("{:?}", data_res.err());
                continue;
            }

            let (tx, network) = data_res.unwrap();

            let receipt_res = self.submit_transaction(Bytes::from(tx), network).await;
            if receipt_res.is_err() {
                warn!("{:?}", receipt_res.err());
                continue;
            }

            info!("transaction included: {:?}", receipt_res.unwrap());
        }
    }

    async fn submit_transaction(
        &self,
        transaction: Bytes,
        network: Network,
    ) -> Result<TransactionReceipt, Error> {
        let provider = Provider::<Http>::try_from(network.get_endpoint())
            .expect("could not instantiate HTTP Provider");

        let pending_tx = provider.send_raw_transaction(transaction).await?;
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

fn parse_message_data(data: &[u8]) -> Result<(Vec<u8>, Network), Error> {
    if data.len() < 2 {
        return Err(anyhow!("message data too short"));
    }
    let network = Network::from(data[0]);
    Ok((data[1..].to_vec(), network))
}

#[tokio::test]
async fn test_server() {
    use crate::DEFAULT_NYM_CLIENT_ENDPOINT;

    let mut server = Server::new(DEFAULT_NYM_CLIENT_ENDPOINT.to_string())
        .await
        .unwrap();
    server.send_address_request().await.unwrap();
    tokio::spawn(async move {
        server.listen().await;
    });
}
