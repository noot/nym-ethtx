use anyhow::{anyhow, Error};
use ethers::{prelude::*, types::transaction::eip2718::TypedTransaction};
use futures::sink::SinkExt;
use nym_addressing::clients::Recipient;
use nym_websocket::requests::ClientRequest;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tracing::info;

/// Client sends transactions through the Nym mixnet to a Server.
pub struct Client<M: Middleware + 'static> {
    recipient: Recipient,
    ws: WebSocketStream<TcpStream>,
    m: M,
}

impl<M: Middleware + 'static> Client<M> {
    pub async fn new(recipient: Recipient, endpoint: String, m: M) -> Result<Self, Error> {
        let (ws, _) = connect_async(endpoint).await?;

        Ok(Client { recipient, ws, m })
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
        let nym_packet = ClientRequest::Send {
            recipient: self.recipient,
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

#[tokio::test]
async fn test_client() {
    use crate::{DEFAULT_NYM_CLIENT_ENDPOINT, DEFAULT_SERVER};
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

    let recipient = Recipient::try_from_base58_string(DEFAULT_SERVER).unwrap();

    let mut client = Client::new(recipient, DEFAULT_NYM_CLIENT_ENDPOINT.to_string(), client)
        .await
        .unwrap();
    let mut tx_req = TypedTransaction::Legacy(TransactionRequest::default());

    let tx_signed = client.sign_transaction_request(&mut tx_req).await.unwrap();
    client.submit_transaction(tx_signed).await.unwrap();
    client.close().await.unwrap();
}
