use super::classes::OperationDomain;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DomainResult<T> {
    pub domain: OperationDomain,
    pub value: T,
}

impl<T: PartialEq> PartialEq for DomainResult<T> {
    fn eq(&self, other: &Self) -> bool {
        self.domain.eq(&other.domain) && self.value.eq(&other.value)
    }
}
