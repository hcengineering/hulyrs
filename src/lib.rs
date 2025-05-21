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

pub mod config;
pub mod services;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ServiceError: {0}")]
    ServiceError(#[from] services::Status),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),

    #[cfg(feature = "kafka")]
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
