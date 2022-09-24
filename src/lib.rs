use anyhow::{anyhow, Error};

pub enum Network {
    Mainnet,
    Goerli,
    Development,
}

impl Network {
    pub fn get_endpoint(&self) -> String {
        match self {
            Network::Mainnet => {
                "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27".to_string()
            }
            Network::Goerli => {
                "https://goerli.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27".to_string()
            }
            Network::Development => "http://localhost:8545".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "goerli" => Ok(Network::Goerli),

            "development" => Ok(Network::Development),
            _ => Err(anyhow!("invalid network {:?}", s)),
        }
    }
}

pub const DEFAULT_NYM_CLIENT_ENDPOINT: &str = "ws://localhost:1977";

// pub mod client;
// pub mod server;
