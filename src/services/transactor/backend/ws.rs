use crate::services::core::WorkspaceUuid;
use crate::services::rpc::util::OkResponse;
use crate::services::rpc::{HelloRequest, HelloResponse, ReqId, Request, Response};
use crate::services::transactor::backend::Backend;
use crate::services::transactor::methods::Method;
use crate::services::{Status, TokenProvider};
use crate::{Error, Result};
use bytes::Bytes;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use reqwest::Client;
use reqwest_websocket::{Message, RequestBuilderExt, WebSocket};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::{broadcast, oneshot};
use tokio::task::JoinHandle;
use tokio_with_wasm::alias as tokio;
use tracing::{error, trace, warn};
use url::Url;
#[cfg(target_family = "wasm")]
pub use wasmtimer::{std::Instant, tokio::sleep, tokio::timeout};
#[cfg(not(target_family = "wasm"))]
use {std::time::Instant, tokio::time::sleep, tokio::time::timeout};

const PONG: &str = "pong!";

enum Command {
    Call {
        payload: Value,
        reply_tx: oneshot::Sender<std::result::Result<OkResponse<Value>, Status>>,
    },
    // TODO: Manual close
    #[allow(dead_code)]
    Close,
}

async fn socket_task(
    mut write: SplitSink<WebSocket, Message>,
    mut read: SplitStream<WebSocket>,
    mut cmd_rx: mpsc::UnboundedReceiver<Command>,
    opts: WsBackendOpts,
    hello_tx: oneshot::Sender<Result<()>>,
    tx_broadcast: broadcast::Sender<Value>,
) -> Result<()> {
    let mut pending =
        HashMap::<ReqId, oneshot::Sender<std::result::Result<OkResponse<Value>, Status>>>::new();
    let mut binary_mode = opts.binary;
    let mut use_compression = opts.compression;
    let next_id = AtomicI32::new(1);

    let hello = HelloRequest {
        request: Request {
            id: Some(ReqId::Num(-1)),
            method: Method::Hello.camel().to_string(),
            params: Vec::new(),
            time: None,
        },
        binary: Some(binary_mode),
        compression: Some(use_compression),
    };
    trace!(target: "ws", ?hello, "sending HELLO");
    write.send(encode_message(&hello, binary_mode)?).await?;

    let mut hello_tx = Some(hello_tx);
    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => match cmd {
                Command::Call { mut payload, reply_tx } => {
                    let id = next_id.fetch_add(1, Ordering::Relaxed);
                    payload["id"] = Value::Number(id.into());

                    pending.insert(id.into(), reply_tx);
                    write.send(encode_message(&payload, binary_mode)?).await?;
                }
                Command::Close => break,
            },

            Some(message) = read.next() => {
                trace!(target: "ws", ?message, "Got message");

                let response: Response<Value>;
                let payload: Bytes;
                match message? {
                    Message::Text(resp) => {
                        // Ping responses don't follow the same structure
                        if resp == PONG {
                            response = Response {
                                result: Some(Value::String(PONG.to_string())),
                                ..Default::default()
                            }
                        } else {
                            response = serde_json::from_str(&resp)?;
                        }

                        payload = resp.into();
                    },
                    Message::Binary(resp) => {
                        if resp == PONG.as_bytes() {
                            response = Response {
                                result: Some(Value::String(PONG.to_string())),
                                ..Default::default()
                            }
                        } else {
                            response = serde_json::from_slice(&resp)?;
                        }

                        payload = resp;
                    },
                    Message::Ping(payload) => {
                        trace!(target: "ws", ?payload, "Received ping, replying...");
                        write.send(encode_message(&Method::Ping.camel(), binary_mode)?).await?;
                        continue;
                    },
                    Message::Close { .. } => break,
                    _ => continue,
                }

                if response.result.as_ref().is_some_and(|v| v == "ping") {
                    trace!(target: "ws", ?payload, "Received ping, replying...");
                    write.send(encode_message(&Method::Ping.camel(), binary_mode)?).await?;
                    continue;
                }

                if matches!(response.id, Some(ReqId::Num(-1))) {
                    if response.result.is_none() && response.error.is_some() {
                        let result = response.into_result();
                        error!(target: "ws", ?result);
                        continue;
                    }

                    if response.result.is_some_and(|result| result == "hello") {
                        // Just ignore any extra HELLOs
                        let Some(hello_tx) = hello_tx.take() else {
                            continue;
                        };

                        let hello = serde_json::from_slice::<HelloResponse>(&payload)?;
                        binary_mode = hello.binary;

                        // TODO: compression support
                        #[allow(unused_assignments)]
                        {
                            use_compression = hello.use_compression.unwrap_or(false);
                        }

                        let _ = hello_tx.send(Ok(()));
                        continue;
                    }

                    continue;
                }

                trace!(target: "ws", ?response, "Full response");
                if let Some(id) = &response.id
                    && let Some(tx) = pending.remove(id) {
                        let _ = tx.send(response.into_result()).ok();
                        continue;
                    }

                if let Some(result) = response.result {
                    match serde_json::from_value::<Vec<Value>>(result) {
                        Ok(tx_array) => {
                            for tx in tx_array {
                                let _ = tx_broadcast.send(tx);
                            }
                        }
                        Err(e) => {
                            warn!(target: "ws", "Failed to deserialize transaction array: {}", e);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn ping_task(cmd_tx: UnboundedSender<Command>) -> Result<()> {
    const PING_TIMEOUT: Duration = Duration::from_secs(10);
    const HANG_TIMEOUT: Duration = Duration::from_secs(60 * 5);

    let mut last_ping_response = None;

    loop {
        sleep(PING_TIMEOUT).await;

        let Some(ping_response_time) = last_ping_response.take() else {
            trace!(target: "ws", "Pinging server");

            let payload = Request {
                id: None,
                method: Method::Ping.camel().to_string(),
                params: Vec::<()>::new(),
                time: None,
            };

            let _response: Value = send_and_wait(&cmd_tx, payload).await?;
            last_ping_response = Some(Instant::now());
            continue;
        };

        if ping_response_time.elapsed() > HANG_TIMEOUT {
            error!("No ping response from server, closing socket");
        }

        last_ping_response = None;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct WsBackendOpts {
    pub binary: bool,
    pub compression: bool,
    /// How long to wait for the server's HELLO response before timing out
    pub hello_timeout: Duration,
}

impl Default for WsBackendOpts {
    fn default() -> Self {
        Self {
            binary: false,
            compression: false,
            hello_timeout: Duration::from_secs(10),
        }
    }
}

struct WsBackendInner {
    workspace: WorkspaceUuid,
    token: SecretString,

    cmd_tx: UnboundedSender<Command>,
    base: Url,
    tx_broadcast: broadcast::Sender<Value>,
    _handle: JoinHandle<()>,
}

#[derive(Clone)]
pub struct WsBackend {
    inner: Arc<WsBackendInner>,
}

impl WsBackend {
    pub(in crate::services::transactor) async fn connect(
        base: Url,
        workspace: WorkspaceUuid,
        token: impl Into<SecretString>,
        opts: WsBackendOpts,
    ) -> Result<Self> {
        let token = token.into();

        let url = base.join(token.expose_secret())?;
        let resp = Client::default()
            .get(url)
            .bearer_auth(token.expose_secret())
            .upgrade()
            .send()
            .await?;
        let ws = resp.into_websocket().await?;

        let (write, read) = ws.split();
        let (hello_tx, hello_rx) = oneshot::channel();

        let (tx_broadcast, _) = broadcast::channel::<Value>(128);

        let tx_broadcast_clone = tx_broadcast.clone();
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<Command>();
        let socket_handle = async move {
            if let Err(e) =
                socket_task(write, read, cmd_rx, opts, hello_tx, tx_broadcast_clone).await
            {
                warn!(target:"ws", ?e, "socket task crashed");
            }
        };

        let cmd_tx2 = cmd_tx.clone();
        let ping_handle = async move {
            if let Err(e) = ping_task(cmd_tx2).await {
                warn!(target:"ws", ?e, "ping task ended");
            }
        };

        let handle = tokio::task::spawn(async move {
            tokio::select! {
                _ = socket_handle => {},
                _ = ping_handle => {},
            }
        });

        match timeout(opts.hello_timeout, hello_rx).await {
            Ok(Ok(Ok(()))) => {}
            Ok(Ok(Err(e))) => return Err(e),
            Err(_) => return Err(Error::Other("timed out waiting for HELLO")),
            _ => return Err(Error::Other("HELLO channel closed unexpectedly")),
        }

        Ok(Self {
            inner: Arc::new(WsBackendInner {
                workspace,
                base,
                cmd_tx,
                tx_broadcast,
                _handle: handle,
                token,
            }),
        })
    }

    pub(in crate::services::transactor) fn tx_stream(
        &self,
    ) -> tokio_stream::wrappers::BroadcastStream<Value> {
        self.inner.tx_broadcast.subscribe().into()
    }
}

fn encode_message<Q: Serialize>(value: &Q, binary_mode: bool) -> Result<Message> {
    if binary_mode {
        Ok(Message::Binary(serde_json::to_vec(value)?.into()))
    } else {
        Ok(Message::Text(serde_json::to_string(value)?))
    }
}

impl TokenProvider for WsBackend {
    fn provide_token(&self) -> Option<&str> {
        Some(self.inner.token.expose_secret())
    }
}

impl Backend for WsBackend {
    async fn get<T: DeserializeOwned + Send>(
        &self,
        method: Method,
        params: impl IntoIterator<Item = (String, Value)>,
    ) -> Result<T> {
        let param_values = params.into_iter().map(|(_k, v)| v).collect::<Vec<_>>();

        let payload = Request {
            id: None,
            method: method.camel().to_string(),
            params: param_values,
            time: None,
        };

        send_and_wait(&self.inner.cmd_tx, payload).await
    }

    async fn post<T: DeserializeOwned + Send, Q: Serialize>(
        &self,
        method: Method,
        body: &Q,
    ) -> Result<T> {
        let Value::Object(body_json) = serde_json::to_value(body)? else {
            return Err(Error::Other("Expected a JSON object"));
        };

        let payload = Request {
            id: None,
            method: method.camel().to_string(),
            params: body_json.values().collect(),
            time: None,
        };

        send_and_wait(&self.inner.cmd_tx, payload).await
    }

    fn base(&self) -> &Url {
        &self.inner.base
    }

    fn workspace(&self) -> WorkspaceUuid {
        self.inner.workspace
    }
}

async fn send_and_wait<T: DeserializeOwned + Send, U: Serialize + Debug>(
    cmd_tx: &UnboundedSender<Command>,
    payload: Request<U>,
) -> Result<T> {
    let payload = serde_json::to_value(&payload)?;
    trace!(target: "ws", %payload, "Sending message");

    let (reply_tx, reply_rx) = oneshot::channel();
    cmd_tx.send(Command::Call { payload, reply_tx }).ok();

    let Ok(reply) = reply_rx.await else {
        return Err(Error::Other("connection closed before reply"));
    };

    let reply = reply?;
    let Some(result) = reply.result else {
        return Err(Error::Other("server didn't return a result"));
    };

    serde_json::from_value(result).map_err(|e| e.into())
}
