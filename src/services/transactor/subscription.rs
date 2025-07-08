use crate::{Error, Result};
use crate::services::core::FindResult;
use crate::services::core::tx::Tx;
use crate::services::transactor::TransactorClient;
use crate::services::transactor::backend::ws::WsBackend;
use crate::services::transactor::document::FindOptions;
use crate::services::transactor::methods::Method;
use futures::Stream;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_with_wasm::alias::sync::broadcast::Receiver;
use tokio_with_wasm::alias::sync::broadcast::error::TryRecvError;
use tokio_with_wasm::alias::task::{self, JoinHandle};

enum SubscriptionState<T: DeserializeOwned> {
    Initial,
    Fetching(JoinHandle<Result<FindResult<T>>>),
    Draining,
    Waiting,
}

pub struct SubscribedQuery<Q: Serialize, T: DeserializeOwned> {
    class: String,
    query: Q,
    options: FindOptions,
    client: TransactorClient<WsBackend>,

    state: SubscriptionState<T>,
    items: VecDeque<T>,
    tx_rx: Receiver<Tx>,
}

impl<Q: Serialize + Clone, T: DeserializeOwned> SubscribedQuery<Q, T> {
    pub fn new(
        client: TransactorClient<WsBackend>,
        class: &str,
        query: Q,
        options: FindOptions,
    ) -> Self {
        let tx_rx = client.backend().tx_stream();

        Self {
            client,
            class: class.to_string(),
            query,
            options,
            state: SubscriptionState::Initial,
            items: VecDeque::new(),
            tx_rx,
        }
    }
}

impl<
    Q: Serialize + Clone + Unpin + Send + Sync + 'static,
    T: DeserializeOwned + Send + Unpin + 'static,
> Stream for SubscribedQuery<Q, T>
{
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.state {
                SubscriptionState::Initial => {
                    let client = self.client.clone();
                    let class = self.class.clone();
                    let query = self.query.clone();
                    let options = self.options.clone();

                    let handle = task::spawn(async move {
                        client
                            .get(
                                Method::FindAll,
                                vec![
                                    ("class", class.into()),
                                    ("query", serde_json::to_value(query)?),
                                    ("options", serde_json::to_value(options)?),
                                ],
                            )
                            .await
                    });

                    self.state = SubscriptionState::Fetching(handle);
                }
                SubscriptionState::Fetching(ref mut handle) => {
                    match Pin::new(handle).poll(cx) {
                        Poll::Ready(Ok(Ok(find_result))) => {
                            self.items = find_result.value.into();
                            self.state = SubscriptionState::Draining;
                            continue;
                        }
                        Poll::Ready(Ok(Err(e))) => {
                            self.state = SubscriptionState::Waiting;
                            return Poll::Ready(Some(Err(e)));
                        }
                        // Task panic
                        Poll::Ready(Err(_join_err)) => {
                            self.state = SubscriptionState::Waiting;
                            return Poll::Ready(Some(Err(Error::SubscriptionFailed)));
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                }
                SubscriptionState::Draining => {
                    let Some(item) = self.items.pop_front() else {
                        self.state = SubscriptionState::Waiting;
                        continue;
                    };

                    return Poll::Ready(Some(Ok(item)));
                }
                SubscriptionState::Waiting => match self.tx_rx.try_recv() {
                    Ok(tx) => {
                        if tx.parent.obj.class != self.class {
                            continue;
                        }

                        self.state = SubscriptionState::Initial;
                    }
                    Err(TryRecvError::Lagged(_)) => {
                        self.state = SubscriptionState::Initial;
                        continue;
                    }
                    Err(TryRecvError::Closed) => {
                        return Poll::Ready(None);
                    }
                    Err(TryRecvError::Empty) => {
                        return Poll::Pending;
                    }
                },
            }
        }
    }
}
