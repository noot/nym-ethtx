use ethers::{prelude::*, types::transaction::eip2718::TypedTransaction};
use nym_addressing::clients::Recipient;
use std::{fs, str::FromStr, sync::Arc};
use structopt::StructOpt;
use tracing_subscriber::EnvFilter;

use nym_ethtx::{client::Client, Network, DEFAULT_NYM_CLIENT_ENDPOINT, DEFAULT_SERVER};

#[derive(StructOpt)]
struct Options {
    /// Nym websocket client endpoint. Default: ws://localhost:1977
    #[structopt(short, long, default_value = DEFAULT_NYM_CLIENT_ENDPOINT)]
    endpoint: String,

    /// Log level. One of debug, info, warn, or error
    #[structopt(short, long, default_value = "info")]
    log: String,

    /// Ethereum network to use.
    /// One of mainnet, goerli, or development.
    #[structopt(short, long, default_value = "development")]
    network: String,

    /// Path to private key file.
    #[structopt(short, long, default_value = "client.key")]
    key: String,

    /// Nym server to send transaction to.
    #[structopt(short, long, default_value = DEFAULT_SERVER)]
    server: String,

    /// Transaction recipient.
    /// Do not set for contract deployment.
    #[structopt(long)]
    to: Option<String>,

    /// Transaction value (in ether).
    #[structopt(long)]
    value: Option<String>,

    /// Transaction gas limit.
    #[structopt(long)]
    gas: Option<String>,

    /// Transaction gas price (in gwei).
    #[structopt(long)]
    gas_price: Option<String>,

    /// Transaction data, hex-encoded.
    #[structopt(long)]
    data: Option<String>,
}

#[tokio::main]
async fn main() {
    let options: Options = Options::from_args();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(options.log)),
        )
        .init();

    let network = Network::from_str(&options.network).unwrap();
    let eth_endpoint = network.get_endpoint();
    let provider =
        Provider::<Http>::try_from(eth_endpoint).expect("could not instantiate HTTP Provider");

    let private_key = fs::read_to_string(&options.key)
        .expect(&format!("cannot read key file {:?}", &options.key));
    let wallet = LocalWallet::from_str(&private_key).unwrap();
    let provider = Arc::new(
        SignerMiddleware::new_with_provider_chain(provider, wallet)
            .await
            .unwrap(),
    );

    let recipient = Recipient::try_from_base58_string(options.server).unwrap();

    let mut client = Client::new(recipient, options.endpoint, provider)
        .await
        .unwrap();

    // form transaction
    let mut tx_req = TransactionRequest::default();

    if let Some(to) = options.to {
        if to[..2].eq("0x") {
            tx_req.to = Some(NameOrAddress::from(H160::from_str(&to).unwrap()));
        } else if to[to.len() - 4..].eq(".eth") {
            // the input is an ENS name
            tx_req.to = Some(NameOrAddress::Name(to));
        } else {
            panic!("invalid address input: must start with 0x or be an ENS name");
        }
    }

    if let Some(value) = options.value {
        tx_req.value = Some(ethers::utils::parse_ether(value).unwrap());
    }

    if let Some(gas) = options.gas {
        tx_req.gas = Some(U256::from_str(&gas).unwrap());
    }

    if let Some(gas_price) = options.gas_price {
        tx_req.gas_price = Some(ethers::utils::parse_units(gas_price, "gwei").unwrap());
    }

    if let Some(data) = options.data {
        tx_req.data = Some(Bytes::from_str(&data).unwrap());
    }

    let mut tx = TypedTransaction::Legacy(tx_req);

    // sign and submit tx
    let tx_signed = client.sign_transaction_request(&mut tx).await.unwrap();
    client.submit_transaction(tx_signed, network).await.unwrap();
    client.close().await.unwrap();
}
