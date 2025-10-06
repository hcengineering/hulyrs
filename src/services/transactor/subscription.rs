use crate::services::core::storage::WithLookup;
use crate::services::core::tx::{TxCreateDoc, TxRemoveDoc, TxUpdateDoc};
use crate::services::event::{Class, Event};
use crate::services::transactor::TransactorClient;
use crate::services::transactor::backend::ws::WsBackend;
use crate::services::transactor::document::{DocumentClient, FindOptions};
use crate::{Error, Result};
use futures::StreamExt;
use futures::{Stream, TryStreamExt};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

pub struct SubscribedQuery<C: Class> {
    tx_rx: BroadcastStream<Value>,
    _phantom: PhantomData<C>,
}

impl<C: Class> SubscribedQuery<C> {
    pub fn new(client: TransactorClient<WsBackend>) -> Self {
        let tx_rx = client.backend().tx_stream();

        Self {
            tx_rx,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TxEvent<C> {
    Created(Box<TxCreateDoc<C>>),
    Updated(Box<TxUpdateDoc<C>>),
    Deleted(Box<TxRemoveDoc>),
}

impl<T> TxEvent<WithLookup<T>> {
    pub fn strip_lookup(self) -> TxEvent<T> {
        match self {
            TxEvent::Created(tx) => TxEvent::Created(Box::new(TxCreateDoc {
                txcud: tx.txcud,
                attributes: tx.attributes.doc,
            })),
            TxEvent::Updated(tx) => TxEvent::Updated(Box::new(TxUpdateDoc {
                txcud: tx.txcud,
                operations: tx.operations,
                retrieve: tx.retrieve,
                _phantom: Default::default(),
            })),
            TxEvent::Deleted(tx) => TxEvent::Deleted(Box::new(TxRemoveDoc { txcud: tx.txcud })),
        }
    }
}

impl<C: Class + DeserializeOwned + Send + Unpin + 'static> Stream for SubscribedQuery<C> {
    type Item = Result<TxEvent<C>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.tx_rx.try_poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(value))) => {
                    if TxCreateDoc::<C>::matches(&value) {
                        let tx: TxCreateDoc<C> = serde_json::from_value(value)?;
                        return Poll::Ready(Some(Ok(TxEvent::Created(Box::new(tx)))));
                    } else if TxUpdateDoc::<C>::matches(&value) {
                        let tx: TxUpdateDoc<C> = serde_json::from_value(value)?;
                        return Poll::Ready(Some(Ok(TxEvent::Updated(Box::new(tx)))));
                    } else if TxRemoveDoc::matches(&value)
                        && value.get("objectClass").and_then(|v| v.as_str()) == Some(C::CLASS)
                    {
                        let tx: TxRemoveDoc = serde_json::from_value(value)?;
                        return Poll::Ready(Some(Ok(TxEvent::Deleted(Box::new(tx)))));
                    }

                    continue;
                }
                Poll::Ready(Some(Err(BroadcastStreamRecvError::Lagged(_)))) => {
                    return Poll::Ready(Some(Err(Error::SubscriptionLagged)));
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum LiveQueryEvent<C> {
    Initial(Vec<C>),
    Polled(TxEvent<C>),
}

impl<T> LiveQueryEvent<WithLookup<T>> {
    pub fn strip_lookup(self) -> LiveQueryEvent<T> {
        match self {
            LiveQueryEvent::Initial(v) => {
                LiveQueryEvent::Initial(v.into_iter().map(WithLookup::into_inner).collect())
            }
            LiveQueryEvent::Polled(v) => LiveQueryEvent::Polled(v.strip_lookup()),
        }
    }
}

pub(super) fn live_query<
    C: Class + Debug + DeserializeOwned + Send + Unpin + 'static,
    Q: Serialize + Send,
>(
    client: TransactorClient<WsBackend>,
    query: Q,
    options: FindOptions,
) -> impl Stream<Item = Result<LiveQueryEvent<C>>> + Send {
    let client_clone = client.clone();
    let initial_fetch = async move {
        let results = client_clone
            .find_all::<Q, C>(C::CLASS, query, &options)
            .await?;
        Ok(LiveQueryEvent::Initial(results.value))
    };

    let event_stream = SubscribedQuery::<C>::new(client).map_ok(LiveQueryEvent::Polled);
    futures::stream::once(initial_fetch).chain(event_stream)
}
