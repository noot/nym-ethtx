use structopt::StructOpt;
use tracing_subscriber::EnvFilter;

use nym_ethtx::{server::Server, DEFAULT_NYM_CLIENT_ENDPOINT};

#[derive(StructOpt)]
struct Options {
    /// Nym websocket client endpoint. Default: ws://localhost:1977
    #[structopt(short, long, default_value = DEFAULT_NYM_CLIENT_ENDPOINT)]
    endpoint: String,

    /// Log level. One of debug, info, warn, or error
    #[structopt(short, long, default_value = "info")]
    log: String,
}

#[tokio::main]
async fn main() {
    let options: Options = Options::from_args();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(options.log)),
        )
        .init();

    let mut server = Server::new(options.endpoint).await.unwrap();
    server.send_address_request().await.unwrap();
    server.listen().await;
}
