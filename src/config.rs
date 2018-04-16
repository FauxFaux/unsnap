#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub server: Server,
    pub keys: Keys,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    pub hostname: String,

    #[serde(default = "default_port")]
    pub port: u16,

    pub nick: String,
    pub user: Option<String>,
    pub real_name: Option<String>,

    pub channels: Vec<String>,
}

fn default_port() -> u16 {
    6697
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keys {
    pub imgur_client_id: String,
}
