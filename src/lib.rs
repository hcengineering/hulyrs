//
// Copyright © 2025 Hardcore Engineering Inc.
//
// Licensed under the Eclipse Public License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License. You may
// obtain a copy of the License at https://www.eclipse.org/legal/epl-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::sync::LazyLock;

pub use reqwest::StatusCode;
use serde::Deserialize;
use serde_with::{StringWithSeparator, formats::CommaSeparator, serde_as};
use url::Url;

pub mod services;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ServiceError: {0}")]
    ServiceError(#[from] services::Status),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error(transparent)]
    Kafka(#[from] rdkafka::error::KafkaError),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error("{0}")]
    HttpError(reqwest::StatusCode, String),

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("{0}")]
    Other(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Config {
    pub token_secret: String,
    pub account_service: Url,
    pub kvs_service: Url,

    #[serde(rename = "kafka_bootstrap")]
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    pub kafka_bootstrap_servers: Vec<String>,

    #[serde(default, rename = "rdkafka_debug")]
    pub kafka_rdkafka_debug: Option<String>,

    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    pub external_regions: Vec<String>,
}

impl Config {
    pub fn kafka_bootstrap_servers(&self) -> String {
        self.kafka_bootstrap_servers.join(",")
    }
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    const DEFAULTS: &str = r#"
        token_secret = "secret"
        account_service = "http://localhost:8080/account"
        kvs_service = "http://localhost:8094"
        kafka_bootstrap = "localhost:19092"
        external_regions = ""
    "#;

    let builder = config::Config::builder()
        .add_source(config::File::from_str(DEFAULTS, config::FileFormat::Toml));

    let config = builder
        .add_source(config::Environment::with_prefix("HULY"))
        .build()
        .and_then(|c| c.try_deserialize::<Config>());

    match config {
        Ok(config) => config,
        Err(error) => {
            eprintln!("configuration error: {}", error);
            std::process::exit(1);
        }
    }
});
