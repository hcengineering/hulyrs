use crate::Result;
use crate::services::core::WorkspaceUuid;
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

trait HttpMethod {
    fn path(&self, workspace: WorkspaceUuid) -> String;
}

impl HttpMethod for Method {
    fn path(&self, workspace: WorkspaceUuid) -> String {
        format!("/api/v1/{}/{}", self.kebab(), workspace)
    }
}

impl super::Backend for HttpBackend {
    async fn get<T: DeserializeOwned + Send>(
        &self,
        method: Method,
        params: impl IntoIterator<Item = (String, Value)>,
    ) -> Result<T> {
        let url = {
            let mut url = self.base().join(&method.path(self.inner.workspace))?;
            let mut qp = url.query_pairs_mut();

            for (name, value) in params {
                match value {
                    Value::String(s) => {
                        qp.append_pair(&name, &s);
                    }
                    _ => {
                        qp.append_pair(&name, &value.to_string());
                    }
                }
            }
            drop(qp);

            url
        };

        <crate::services::HttpClient as JsonClient>::get(&self.inner.client, self, url).await
    }

    async fn post<T: DeserializeOwned + Send, Q: Serialize>(
        &self,
        method: Method,
        body: &Q,
    ) -> Result<T> {
        let url = self.base().join(&method.path(self.inner.workspace))?;
        <crate::services::HttpClient as JsonClient>::post(&self.inner.client, self, url, body).await
    }

    fn base(&self) -> &Url {
        &self.inner.base
    }

    fn workspace(&self) -> WorkspaceUuid {
        self.inner.workspace
    }
}
