use anyhow::{anyhow, Error};
use ethers::{prelude::*, types::transaction::eip2718::TypedTransaction};
use futures::sink::SinkExt;
use nym_addressing::clients::Recipient;
use nym_websocket::requests::ClientRequest;
use std::{fs, str::FromStr};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

use nym_ethtx::DEFAULT_NYM_CLIENT_ENDPOINT;

pub const DEFAULT_SERVER: &str = "DXHLCASnJGSesso5hXus1CtgifBpaPqAj7thZphp52xN.7udbVvZ199futJNur71L3vHDNdnbVxxBvFKVzhEifXvE@5vC8spDvw5VDQ8Zvd9fVvBhbUDv9jABR4cXzd4Kh5vz";

pub struct Client {
    ws: WebSocketStream<TcpStream>,
    wallet: LocalWallet,
}

impl Client {
    pub async fn new(endpoint: Option<String>, wallet: LocalWallet) -> Result<Self, Error> {
        let (ws, _) =
            connect_async(endpoint.unwrap_or(DEFAULT_NYM_CLIENT_ENDPOINT.to_string())).await?;

        Ok(Client { ws, wallet })
    }

    pub fn sign_transaction_request(&self, tx_req: &TypedTransaction) -> Result<Bytes, Error> {
        let sig = self.wallet.sign_transaction_sync(tx_req);
        Ok(tx_req.rlp_signed(&sig))
    }

    pub async fn submit_transaction(&mut self, encoded_tx: Bytes) -> Result<(), Error> {
        let recipient = Recipient::try_from_base58_string(DEFAULT_SERVER)?;

        let nym_packet = ClientRequest::Send {
            recipient,
            message: encoded_tx.to_vec(),
            with_reply_surb: false,
        };

        self.ws
            .send(Message::Binary(nym_packet.serialize()))
            .await
            .map_err(|e| anyhow!("failed to send packet: {:?}", e))
    }

    pub async fn close(&mut self) -> Result<(), Error> {
        self.ws
            .close(None)
            .await
            .map_err(|e| anyhow!("failed to close: {:?}", e))
    }
}

#[tokio::main]
async fn main() {
    let filepath = "client.key";
    let private_key = fs::read_to_string(filepath).expect("cannot read key file");
    let wallet = LocalWallet::from_str(&private_key).unwrap();
    let mut client = Client::new(None, wallet).await.unwrap();

    let tx_req = TypedTransaction::Legacy(TransactionRequest {
        from: None,
        to: Some(NameOrAddress::from(
            "0x1EA777Dc621f5A63E63bbcE4fc9caE3c5CDEDAFB",
        )),
        gas: None,
        gas_price: None,
        value: Some(U256::from(100_000_000)),
        data: None,
        nonce: None,
        chain_id: None,
    });

    let tx_signed = client.sign_transaction_request(&tx_req).unwrap();
    client.submit_transaction(tx_signed).await.unwrap();
    client.close().await.unwrap();
}

#[tokio::test]
async fn test_client() {
    use ethers::utils::Anvil;

    let anvil = Anvil::new().spawn();
    let wallet: LocalWallet = anvil.keys()[0].clone().into();

    let mut client = Client::new(None, wallet).await.unwrap();
    let tx_req = TypedTransaction::Legacy(TransactionRequest {
        from: None,
        to: None,
        gas: None,
        gas_price: None,
        value: None,
        data: None,
        nonce: None,
        chain_id: None,
    });

    let tx_signed = client.sign_transaction_request(&tx_req).unwrap();
    client.submit_transaction(tx_signed).await.unwrap();
    client.close().await.unwrap();
}
