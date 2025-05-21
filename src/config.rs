use serde::{Deserialize, Deserializer};
use serde_with::{DisplayFromStr, StringWithSeparator, formats::CommaSeparator, serde_as};
use std::str::FromStr;
use url::Url;

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Config {
    pub token_secret: String,
    pub account_service: Url,
    pub kvs_service: Url,

    #[serde_as(as = "DisplayFromStr")]
    pub log: tracing::Level,

    #[serde(rename = "kafka_bootstrap")]
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    pub kafka_bootstrap_servers: Vec<String>,

    #[serde(default, rename = "rdkafka_debug")]
    pub kafka_rdkafka_debug: Option<String>,

    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    pub external_regions: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token_secret: String::from("secret"),
            account_service: Url::from_str("http://localhost:8080/account").unwrap(),
            kvs_service: Url::from_str("http://localhost:8094").unwrap(),
            kafka_bootstrap_servers: vec![String::from("localhost:19092")],
            log: tracing::Level::INFO,
            kafka_rdkafka_debug: None,
            external_regions: Vec::new(),
        }
    }
}

impl Config {
    pub fn kafka_bootstrap_servers(&self) -> String {
        self.kafka_bootstrap_servers.join(",")
    }

    pub fn from_env() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::Environment::with_prefix("HULY"))
            .build()
            .and_then(|c| c.try_deserialize::<Config>())
    }
}
