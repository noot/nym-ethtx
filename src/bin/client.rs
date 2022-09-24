use anyhow::{anyhow, Error};
use ethers::{prelude::*, types::transaction::eip2718::TypedTransaction};
use futures::sink::SinkExt;
use nym_addressing::clients::Recipient;
use nym_websocket::requests::ClientRequest;
use std::{fs, str::FromStr, sync::Arc};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tracing::info;
use tracing_subscriber::EnvFilter;

use nym_ethtx::{Network, DEFAULT_NYM_CLIENT_ENDPOINT};

pub const DEFAULT_SERVER: &str = "DXHLCASnJGSesso5hXus1CtgifBpaPqAj7thZphp52xN.7udbVvZ199futJNur71L3vHDNdnbVxxBvFKVzhEifXvE@5vC8spDvw5VDQ8Zvd9fVvBhbUDv9jABR4cXzd4Kh5vz";

/// Client sends transactions through the Nym mixnet to a Server.
pub struct Client<M: Middleware + 'static> {
    ws: WebSocketStream<TcpStream>,
    m: M,
}

impl<M: Middleware + 'static> Client<M> {
    pub async fn new(endpoint: Option<String>, m: M) -> Result<Self, Error> {
        let (ws, _) =
            connect_async(endpoint.unwrap_or(DEFAULT_NYM_CLIENT_ENDPOINT.to_string())).await?;

        Ok(Client { ws, m })
    }

    pub async fn sign_transaction_request(
        &self,
        tx: &mut TypedTransaction,
    ) -> Result<Bytes, Error> {
        self.m.fill_transaction(tx, None).await?;
        let sig = self
            .m
            .sign_transaction(tx, self.m.default_sender().unwrap())
            .await?;
        info!("signed transaction {:?}", tx.hash(&sig));
        Ok(tx.rlp_signed(&sig))
    }

    pub async fn submit_transaction(&mut self, tx: Bytes) -> Result<(), Error> {
        let recipient = Recipient::try_from_base58_string(DEFAULT_SERVER)?;
        let nym_packet = ClientRequest::Send {
            recipient,
            message: tx.to_vec(),
            with_reply_surb: false,
        };

        self.ws
            .send(Message::Binary(nym_packet.serialize()))
            .await
            .map_err(|e| anyhow!("failed to send packet: {:?}", e))?;
        Ok(())
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
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let eth_endpoint = Network::Development.get_endpoint();
    let provider =
        Provider::<Http>::try_from(eth_endpoint).expect("could not instantiate HTTP Provider");

    let filepath = "client.key";
    let private_key = fs::read_to_string(filepath).expect("cannot read key file");
    let wallet = LocalWallet::from_str(&private_key).unwrap();
    let provider = Arc::new(
        SignerMiddleware::new_with_provider_chain(provider, wallet)
            .await
            .unwrap(),
    );
    let mut client = Client::new(None, provider).await.unwrap();

    let mut tx_req = TypedTransaction::Legacy(TransactionRequest {
        from: None,
        to: Some(NameOrAddress::from(
            H160::from_str("0x1EA777Dc621f5A63E63bbcE4fc9caE3c5CDEDAFB").unwrap(),
        )),
        gas: None,
        gas_price: None,
        value: Some(U256::from(100_000_000)),
        data: None,
        nonce: None,
        chain_id: None,
    });

    let tx_signed = client.sign_transaction_request(&mut tx_req).await.unwrap();
    client.submit_transaction(tx_signed).await.unwrap();
    client.close().await.unwrap();
}

#[tokio::test]
async fn test_client() {
    use ethers::utils::Anvil;
    use std::sync::Arc;
    use std::time::Duration;

    let anvil = Anvil::new().spawn();
    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let provider = Provider::<Http>::try_from(anvil.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64));
    let client = Arc::new(
        SignerMiddleware::new_with_provider_chain(provider, wallet)
            .await
            .unwrap(),
    );

    let mut client = Client::new(None, client).await.unwrap();
    let mut tx_req = TypedTransaction::Legacy(TransactionRequest {
        from: None,
        to: None,
        gas: None,
        gas_price: None,
        value: None,
        data: None,
        nonce: None,
        chain_id: None,
    });

    let tx_signed = client.sign_transaction_request(&mut tx_req).await.unwrap();
    client.submit_transaction(tx_signed).await.unwrap();
    client.close().await.unwrap();
}
