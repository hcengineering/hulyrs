use crate::services::Status;
use crate::services::rpc::{Chunk, RateLimitInfo, ReqId, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkResponse<R> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<R>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ReqId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk: Option<Chunk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bfst: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<u32>,
}

impl<R> Response<R> {
    pub fn into_result(self) -> Result<OkResponse<R>, Status> {
        match self.error {
            Some(e) => Err(e),
            None => Ok(OkResponse {
                result: self.result,
                id: self.id,
                terminate: self.terminate,
                rate_limit: self.rate_limit,
                chunk: self.chunk,
                time: self.time,
                bfst: self.bfst,
                queue: self.queue,
            }),
        }
    }
}
