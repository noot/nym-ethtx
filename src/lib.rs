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
}

pub const DEFAULT_NYM_CLIENT_ENDPOINT: &str = "ws://localhost:1977";
