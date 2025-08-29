use crate::Result;
use crate::services::TokenProvider;
use crate::services::core::WorkspaceUuid;
use crate::services::core::classes::OperationDomain;
use crate::services::core::storage::DomainResult;
use crate::services::transactor::Transaction;
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

    async fn domain_request<T: DeserializeOwned + Send, Q: Serialize>(
        &self,
        domain: OperationDomain,
        operation: &str,
        params: &Q,
    ) -> Result<DomainResult<T>>;

    async fn tx_raw<T: Serialize, R: DeserializeOwned + Send>(&self, tx: T) -> Result<R>;

    async fn tx<T: Transaction, R: DeserializeOwned + Send>(&self, tx: T) -> Result<R> {
        self.tx_raw(tx.to_value()?).await
    }

    fn base(&self) -> &Url;

    fn workspace(&self) -> WorkspaceUuid;
}
