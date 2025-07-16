use crate::services::event::Event;
use crate::services::transactor::TransactorClient;
use crate::services::transactor::backend::ws::WsBackend;
use crate::{Error, Result};
use futures::{Stream, TryStreamExt};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

pub struct SubscribedQuery<T: Event + DeserializeOwned> {
    tx_rx: BroadcastStream<Value>,
    _phantom: PhantomData<T>,
}

impl<T: Event + DeserializeOwned> SubscribedQuery<T> {
    pub fn new(client: TransactorClient<WsBackend>) -> Self {
        let tx_rx = client.backend().tx_stream();

        Self {
            tx_rx,
            _phantom: PhantomData,
        }
    }
}

impl<T: Event + DeserializeOwned + Send + Unpin + 'static> Stream for SubscribedQuery<T> {
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.tx_rx.try_poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(value))) => {
                    if T::matches(&value) {
                        let event = serde_json::from_value(value).map_err(Error::from);
                        return Poll::Ready(Some(event));
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
