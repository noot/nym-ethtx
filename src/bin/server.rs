use ethers::prelude::*;
use structopt::StructOpt;
use tracing_subscriber::EnvFilter;

use nym_ethtx::{server::Server, Network, DEFAULT_NYM_CLIENT_ENDPOINT};

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
