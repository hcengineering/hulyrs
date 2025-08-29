use super::classes::OperationDomain;
use crate::services::core::classes::Ref;
use crate::services::core::ser::Data;
use crate::services::event::{Class, Event};
use crate::services::transactor::tx::Doc;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

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

impl<T> Class for TxWorkspaceEvent<T> {
    const CLASS: &'static str = crate::services::core::class::TxWorkspaceEvent;
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

impl<T> Class for TxDomainEvent<T> {
    const CLASS: &'static str = crate::services::core::class::TxDomainEvent;
}

impl<T: Class> Event for TxDomainEvent<T> {
    fn matches(value: &Value) -> bool {
        if value.get("_class").and_then(|v| v.as_str()) != Some(Self::CLASS) {
            return false;
        }
        value.get("objectClass").and_then(|v| v.as_str()) == Some(T::CLASS)
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxCreateDoc<T> {
    #[serde(flatten)]
    pub txcud: TxCUD,
    #[serde(flatten)]
    pub attributes: Data<T>,
}

impl<T> Class for TxCreateDoc<T> {
    const CLASS: &'static str = crate::services::core::class::TxCreateDoc;
}

impl<T: Class> Event for TxCreateDoc<T> {
    fn matches(value: &Value) -> bool {
        if value.get("_class").and_then(|v| v.as_str()) != Some(Self::CLASS) {
            return false;
        }
        value.get("objectClass").and_then(|v| v.as_str()) == Some(T::CLASS)
    }
}

impl<'de, T> Deserialize<'de> for TxCreateDoc<T>
where
    T: Serialize + DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let txcud = serde_json::from_value(value.clone()).map_err(serde::de::Error::custom)?;

        let attributes = serde_json::from_value(value).map_err(serde::de::Error::custom)?;

        Ok(TxCreateDoc { txcud, attributes })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxRemoveDoc {
    #[serde(flatten)]
    pub txcud: TxCUD,
}

impl Class for TxRemoveDoc {
    const CLASS: &'static str = crate::services::core::class::TxRemoveDoc;
}

impl Event for TxRemoveDoc {}
