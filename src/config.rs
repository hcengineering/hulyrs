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

use derive_builder::Builder;
use secrecy::SecretString;
use serde::Deserialize;
use serde_with::{DisplayFromStr, StringWithSeparator, formats::CommaSeparator, serde_as};
use url::Url;

use crate::Result;

#[serde_as]
#[derive(Deserialize, Debug, Builder, Clone)]
pub struct Config {
    #[builder(setter(strip_option, into), default)]
    pub token_secret: Option<SecretString>,

    #[builder(setter(strip_option, into), default)]
    pub account_service: Option<Url>,

    #[builder(setter(strip_option, into), default)]
    pub kvs_service: Option<Url>,

    #[serde_as(as = "DisplayFromStr")]
    #[builder(setter(strip_option, into), default = "tracing::Level::INFO")]
    pub log: tracing::Level,

    #[serde(rename = "kafka_bootstrap")]
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    #[builder(default)]
    pub kafka_bootstrap_servers: Vec<String>,

    #[serde(default, rename = "rdkafka_debug")]
    #[builder(setter(strip_option, into), default)]
    pub kafka_rdkafka_debug: Option<String>,

    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    #[builder(default)]
    pub external_regions: Vec<String>,
}

impl Config {
    pub fn kafka_bootstrap_servers(&self) -> String {
        self.kafka_bootstrap_servers.join(",")
    }

    pub fn auto() -> Result<Self> {
        const DEFAULTS: &str = r#"
        token_secret = "secret"
        account_service = "http://localhost:8080/account"
        kvs_service = "http://localhost:8094"
        kafka_bootstrap = "localhost:19092"
        log = "INFO"
        external_regions = ""
    "#;

        let config = config::Config::builder()
            .add_source(config::File::from_str(DEFAULTS, config::FileFormat::Toml))
            .add_source(config::Environment::with_prefix("HULY"))
            .build()
            .and_then(|c| c.try_deserialize::<Config>());

        Ok(config?)
    }
}
