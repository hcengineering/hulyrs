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

use std::num::NonZeroU32;

use derive_builder::Builder;
use secrecy::{ExposeSecret, SecretString};
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

    #[cfg(feature = "reqwest_middleware")]
    #[builder(default = "NonZeroU32::try_from(10).unwrap()")]
    pub account_service_rate_limit: NonZeroU32,

    #[builder(setter(strip_option, into), default)]
    pub kvs_service: Option<Url>,

    #[builder(setter(strip_option, into), default)]
    pub collaborator_service: Option<Url>,

    #[serde_as(as = "DisplayFromStr")]
    #[builder(setter(strip_option, into), default = "tracing::Level::INFO")]
    pub log: tracing::Level,

    #[cfg(feature = "kafka")]
    #[serde(rename = "kafka_bootstrap")]
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    #[builder(default)]
    pub kafka_bootstrap_servers: Vec<String>,

    #[cfg(feature = "kafka")]
    #[serde(default, rename = "rdkafka_debug")]
    #[builder(setter(strip_option, into), default)]
    pub kafka_rdkafka_debug: Option<String>,

    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    #[builder(default)]
    pub external_regions: Vec<String>,

    #[builder(setter(strip_option, into), default)]
    pub pulse_service: Option<Url>,

    #[cfg(feature = "otel")]
    #[serde(default)]
    pub otel_mode: crate::services::otel::OtelMode,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(feature = "kafka")]
        let kafka_eq = self.kafka_bootstrap_servers == other.kafka_bootstrap_servers
            && self.kafka_rdkafka_debug == other.kafka_rdkafka_debug;
        #[cfg(not(feature = "kafka"))]
        let kafka_eq = true;

        #[cfg(feature = "reqwest_middleware")]
        let rate_limit_eq = self.account_service_rate_limit == other.account_service_rate_limit;

        #[cfg(not(feature = "reqwest_middleware"))]
        let rate_limit_eq = true;

        self.token_secret.as_ref().map(SecretString::expose_secret)
            == other.token_secret.as_ref().map(SecretString::expose_secret)
            && self.account_service == other.account_service
            && self.kvs_service == other.kvs_service
            && self.collaborator_service == other.collaborator_service
            && self.log == other.log
            && kafka_eq
            && rate_limit_eq
            && self.external_regions == other.external_regions
            && self.pulse_service == other.pulse_service
    }
}

impl Config {
    #[cfg(feature = "kafka")]
    pub fn kafka_bootstrap_servers(&self) -> String {
        self.kafka_bootstrap_servers.join(",")
    }

    pub fn auto() -> Result<Self> {
        const DEFAULTS: &str = r#"
        token_secret = "secret"
        account_service = "http://localhost:8080/account"
        account_service_rate_limit = 10
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
