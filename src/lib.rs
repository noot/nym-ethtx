use anyhow::{anyhow, Error};

pub mod client;
pub mod server;

pub const DEFAULT_NYM_CLIENT_ENDPOINT: &str = "ws://localhost:1977";
pub const DEFAULT_SERVER: &str = "HGLX5467Kr8hHaYENr8meY3KDH5BozVQRR8XTBD8UseB.Fdnv3igmSrGcUZSA4bUyqa6adyHKjZGyhFnnkWMJsGAt@62Lq9D5yhRVXyeHrBjqoQMg3i9aVTJY7nQSnB74VH31t";

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

    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Network::Mainnet => vec![1],
            Network::Goerli => vec![5],
            Network::Development => vec![255],
        }
    }
}

impl From<u8> for Network {
    fn from(b: u8) -> Self {
        match b {
            1u8 => Network::Mainnet,
            5u8 => Network::Goerli,
            _ => Network::Development,
        }
    }
}
