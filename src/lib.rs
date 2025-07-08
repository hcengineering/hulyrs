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

pub use reqwest::StatusCode;

mod config;
pub mod services;

pub use config::{Config, ConfigBuilder, ConfigBuilderError};
pub use services::ServiceFactory;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ServiceError: {0}")]
    ServiceError(#[from] services::Status),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Ws(#[from] reqwest_websocket::Error),

    #[error(transparent)]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),

    #[cfg(feature = "kafka")]
    #[error(transparent)]
    Kafka(#[from] rdkafka::error::KafkaError),
    
    #[error("Subscription task panicked")]
    SubscriptionFailed,

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error("{0}")]
    HttpError(reqwest::StatusCode, String),

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    Config(#[from] ::config::ConfigError),

    #[error("{0}")]
    Other(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn optional_rounded_float<'de, D, T: num_traits::FromPrimitive>(
    deserializer: D,
) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;

    if let Some(float) = Option::<f64>::deserialize(deserializer)? {
        T::from_f64(float.round()).map(Some).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "Cannot convert {} to {}",
                float,
                std::any::type_name::<T>()
            ))
        })
    } else {
        Ok(None)
    }
}
