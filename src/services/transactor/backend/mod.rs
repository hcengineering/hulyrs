use crate::Result;
use crate::services::TokenProvider;
use crate::services::core::WorkspaceUuid;
use crate::services::transactor::methods::Method;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use url::Url;

pub mod http;
pub mod ws;

#[allow(async_fn_in_trait)]
pub trait Backend: Clone + TokenProvider + 'static {
    async fn get<T: DeserializeOwned + Send>(
        &self,
        method: Method,
        params: impl IntoIterator<Item = (String, Value)>,
    ) -> Result<T>;

    async fn post<T: DeserializeOwned + Send, Q: Serialize>(
        &self,
        method: Method,
        body: &Q,
    ) -> Result<T>;

    fn base(&self) -> &Url;

    fn workspace(&self) -> WorkspaceUuid;
}
