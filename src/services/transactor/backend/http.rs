use crate::Result;
use crate::services::core::WorkspaceUuid;
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
        let mut url = self.base().join(&format!("/api/v1/{}", method.kebab()))?;
        let mut qp = url.query_pairs_mut();
        for (name, value) in params {
            qp.append_pair(&name, &value.to_string());
        }
        drop(qp);

        <crate::services::HttpClient as JsonClient>::get(&self.inner.client, self, url).await
    }

    async fn post<T: DeserializeOwned + Send, Q: Serialize>(
        &self,
        method: Method,
        body: &Q,
    ) -> Result<T> {
        self.post_path(&format!("/api/v1/{}", method.kebab()), body)
            .await
    }

    fn base(&self) -> &Url {
        &self.inner.base
    }

    fn workspace(&self) -> WorkspaceUuid {
        self.inner.workspace
    }
}
