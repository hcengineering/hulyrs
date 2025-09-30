use crate::Result;
use crate::services::core::WorkspaceUuid;
use crate::services::core::classes::OperationDomain;
use crate::services::core::storage::DomainResult;
use crate::services::transactor::backend::Backend;
use crate::services::transactor::methods::Method;
use crate::services::{JsonClient, TokenProvider};
use reqwest_middleware::ClientWithMiddleware;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
use url::Url;

pub type HttpClient = ClientWithMiddleware;

struct HttpBackendInner {
    workspace: WorkspaceUuid,
    base: Url,
    client: HttpClient,
    token: SecretString,
}

#[derive(Clone)]
pub struct HttpBackend {
    inner: Arc<HttpBackendInner>,
}

impl HttpBackend {
    pub fn new(
        client: HttpClient,
        base: Url,
        workspace: WorkspaceUuid,
        token: impl Into<SecretString>,
    ) -> Self {
        Self {
            inner: Arc::new(HttpBackendInner {
                workspace,
                base,
                client,
                token: token.into(),
            }),
        }
    }

    pub(crate) async fn get_path<T: DeserializeOwned + Send>(
        &self,
        path: &str,
        params: impl IntoIterator<Item = (String, Value)>,
    ) -> Result<T> {
        let mut url = self.base().join(path)?;
        {
            let mut qp = url.query_pairs_mut();
            for (name, value) in params {
                if let Value::String(string) = &value {
                    qp.append_pair(&name, string);
                } else {
                    qp.append_pair(&name, &value.to_string());
                }
            }
        }

        <crate::services::HttpClient as JsonClient>::get(&self.inner.client, self, url).await
    }

    pub(crate) async fn post_path<T: DeserializeOwned + Send, Q: Serialize>(
        &self,
        path: &str,
        body: &Q,
    ) -> Result<T> {
        let url = self.base().join(path)?;
        <crate::services::HttpClient as JsonClient>::post(&self.inner.client, self, url, body).await
    }
}

impl JsonClient for HttpBackend {
    fn get<U: TokenProvider, R: DeserializeOwned>(
        &self,
        user: &U,
        url: Url,
    ) -> impl Future<Output = Result<R>> {
        JsonClient::get(&self.inner.client, user, url)
    }

    fn post<U: TokenProvider, Q: Serialize, R: DeserializeOwned>(
        &self,
        user: &U,
        url: Url,
        body: &Q,
    ) -> impl Future<Output = Result<R>> {
        JsonClient::post(&self.inner.client, user, url, body)
    }
}

impl TokenProvider for HttpBackend {
    fn provide_token(&self) -> Option<&str> {
        Some(self.inner.token.expose_secret())
    }
}

impl TokenProvider for &'_ HttpBackend {
    fn provide_token(&self) -> Option<&str> {
        Some(self.inner.token.expose_secret())
    }
}

impl super::Backend for HttpBackend {
    async fn get<T: DeserializeOwned + Send>(
        &self,
        method: Method,
        params: impl IntoIterator<Item = (String, Value)>,
    ) -> Result<T> {
        self.get_path(
            &format!("/api/v1/{}/{}", method.kebab(), self.workspace()),
            params,
        )
        .await
    }

    async fn post<T: DeserializeOwned + Send, Q: Serialize>(
        &self,
        method: Method,
        body: &Q,
    ) -> Result<T> {
        self.post_path(
            &format!("/api/v1/{}/{}", method.kebab(), self.workspace()),
            body,
        )
        .await
    }

    async fn domain_request<T: DeserializeOwned + Send, Q: Serialize>(
        &self,
        domain: OperationDomain,
        operation: &str,
        params: &Q,
    ) -> Result<DomainResult<T>> {
        let params = (String::from("params"), serde_json::to_value(params)?);
        self.get_path(
            &format!(
                "/api/v1/{}/{domain}/{operation}/{}",
                Method::Event.kebab(),
                self.workspace()
            ),
            std::iter::once(params),
        )
        .await
    }

    async fn tx_raw<T: Serialize, R: DeserializeOwned + Send>(&self, tx: T) -> Result<R> {
        self.post_path(&format!("/api/v1/tx/{}", self.workspace()), &tx)
            .await
    }

    fn base(&self) -> &Url {
        &self.inner.base
    }

    fn workspace(&self) -> WorkspaceUuid {
        self.inner.workspace
    }
}
