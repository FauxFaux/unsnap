use serde_derive::Deserialize;
use serde_derive::Serialize;

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
    pub password: Option<String>,
    pub nick_password: Option<String>,

    pub channels: Vec<String>,
}

fn default_port() -> u16 {
    6697
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keys {
    pub imgur_client_id: String,
    pub twitter_app_key: String,
    pub twitter_app_secret: String,
    pub spotify_app_key: String,
    pub spotify_app_secret: String,
    pub youtube_developer_key: String,
}
