use super::classes::OperationDomain;
use crate::services::core::classes::Ref;
use crate::services::event::{Class, Event, HasId};
use crate::services::transactor::tx::Doc;
use derive_builder::Builder;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Tx {
    #[serde(flatten)]
    pub doc: Doc,
    /// The space where the transaction will operate
    pub object_space: Ref,
}

#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceEvent {
    UpgradeScheduled = 0,
    IndexingUpdate = 1,
    SecurityChange = 2,
    MaintenanceNotification = 3,
    BulkUpdate = 4,
    LastTx = 5,
}

impl<'de> Deserialize<'de> for WorkspaceEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WorkspaceEventVisitor;

        impl<'de> serde::de::Visitor<'de> for WorkspaceEventVisitor {
            type Value = WorkspaceEvent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an integer between 0 and 5 for WorkspaceEvent enum")
            }

            fn visit_u8<E>(self, value: u8) -> Result<WorkspaceEvent, E>
            where
                E: serde::de::Error,
            {
                match value {
                    0 => Ok(WorkspaceEvent::UpgradeScheduled),
                    1 => Ok(WorkspaceEvent::IndexingUpdate),
                    2 => Ok(WorkspaceEvent::SecurityChange),
                    3 => Ok(WorkspaceEvent::MaintenanceNotification),
                    4 => Ok(WorkspaceEvent::BulkUpdate),
                    5 => Ok(WorkspaceEvent::LastTx),
                    _ => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Unsigned(value as u64),
                        &self,
                    )),
                }
            }
        }

        deserializer.deserialize_u8(WorkspaceEventVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TxWorkspaceEvent<T> {
    #[serde(flatten)]
    pub tx: Tx,
    pub domain: WorkspaceEvent,
    pub event: T,
}

impl<T: Debug> Class for TxWorkspaceEvent<T> {
    const CLASS: &'static str = crate::services::core::class::TxWorkspaceEvent;
}

impl<T> HasId for TxWorkspaceEvent<T> {
    fn id(&self) -> &str {
        &self.tx.doc.id
    }
}

impl<T: Class> Event for TxWorkspaceEvent<T> {
    fn matches(value: &Value) -> bool {
        if value.get("_class").and_then(|v| v.as_str()) != Some(Self::CLASS) {
            return false;
        }
        value.get("objectClass").and_then(|v| v.as_str()) == Some(T::CLASS)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TxDomainEvent<T> {
    #[serde(flatten)]
    pub tx: Tx,
    pub domain: OperationDomain,
    pub event: T,
}

impl<T: Debug> Class for TxDomainEvent<T> {
    const CLASS: &'static str = crate::services::core::class::TxDomainEvent;
}

impl<T: Debug> Event for TxDomainEvent<T> {
    fn matches(value: &Value) -> bool {
        value.get("_class").and_then(|v| v.as_str()) == Some(Self::CLASS)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxCUD {
    #[serde(flatten)]
    pub tx: Tx,
    pub object_id: Ref,
    pub object_class: Ref,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attached_to: Option<Ref>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attached_to_class: Option<Ref>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxCreateDoc<T> {
    #[serde(flatten)]
    pub txcud: TxCUD,
    #[serde(flatten)]
    pub attributes: T,
}

impl<T: Debug> Class for TxCreateDoc<T> {
    const CLASS: &'static str = crate::services::core::class::TxCreateDoc;
}

impl<C> HasId for TxCreateDoc<C> {
    fn id(&self) -> &str {
        &self.txcud.object_id
    }
}

impl<T: Class> Event for TxCreateDoc<T> {
    fn matches(value: &Value) -> bool {
        if value.get("_class").and_then(|v| v.as_str()) != Some(Self::CLASS) {
            return false;
        }
        value.get("objectClass").and_then(|v| v.as_str()) == Some(T::CLASS)
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull: Option<HashMap<String, Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<HashMap<String, Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub inc: Option<HashMap<String, Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub unset: Option<HashMap<String, Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub space: Option<Ref>,

    #[serde(flatten)]
    pub set_operations: HashMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct TxUpdateDoc<C> {
    #[serde(flatten)]
    pub txcud: TxCUD,

    #[serde(flatten)]
    pub operations: DocumentUpdate,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieve: Option<bool>,

    #[serde(skip)]
    #[builder(setter(skip), default)]
    pub(crate) _phantom: PhantomData<C>,
}

impl<C: Debug> Class for TxUpdateDoc<C> {
    const CLASS: &'static str = crate::services::core::class::TxUpdateDoc;
}

impl<C> HasId for TxUpdateDoc<C> {
    fn id(&self) -> &str {
        &self.txcud.object_id
    }
}

impl<C: Class> Event for TxUpdateDoc<C> {
    fn matches(value: &Value) -> bool {
        if value.get("_class").and_then(|v| v.as_str()) != Some(Self::CLASS) {
            return false;
        }
        value.get("objectClass").and_then(|v| v.as_str()) == Some(C::CLASS)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct TxRemoveDoc<C> {
    #[serde(flatten)]
    pub txcud: TxCUD,

    #[serde(skip)]
    #[builder(setter(skip), default)]
    pub(crate) _phantom: PhantomData<C>,
}

impl<C: Clone> TxRemoveDoc<C> {
    pub fn builder() -> TxRemoveDocBuilder<C> {
        TxRemoveDocBuilder::default()
    }
}

impl<C: Debug> Class for TxRemoveDoc<C> {
    const CLASS: &'static str = crate::services::core::class::TxRemoveDoc;
}

impl<C> HasId for TxRemoveDoc<C> {
    fn id(&self) -> &str {
        &self.txcud.object_id
    }
}

impl<C: Class> Event for TxRemoveDoc<C> {
    fn matches(value: &Value) -> bool {
        if value.get("_class").and_then(|v| v.as_str()) != Some(Self::CLASS) {
            return false;
        }
        value.get("objectClass").and_then(|v| v.as_str()) == Some(C::CLASS)
    }
}
