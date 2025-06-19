//
// Copyright © 2025 Hardcore Engineering Inc.
//
// Licensed under the Eclipse Public License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License. You may
// obtain a copy of the License at https://www.eclipse.org/legal/epl-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//
// See the License for the specific language governing permissions and
// limitations under the License.
//

use serde::{Deserialize, Serialize};

use crate::{
    Result,
    services::{
        HttpClient, JsonClient,
        types::{PersonId, PersonUuid, SocialIdType},
    },
};

#[derive(Serialize, Debug, derive_builder::Builder)]
#[serde(rename_all = "camelCase")]
pub struct EnsurePersonRequest {
    pub social_type: SocialIdType,
    pub social_value: String,

    #[builder(setter(into))]
    pub first_name: String,

    #[builder(setter(into), default)]
    pub last_name: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnsurePersonResponse {
    pub uuid: PersonUuid,
    pub social_id: PersonId,
}

pub trait EnsurePerson {
    fn ensure_person(
        &self,
        request: &EnsurePersonRequest,
    ) -> impl Future<Output = Result<EnsurePersonResponse>>;
}

impl EnsurePerson for super::TransactorClient {
    async fn ensure_person(&self, request: &EnsurePersonRequest) -> Result<EnsurePersonResponse> {
        let path = format!("/api/v1/ensure-person/{}", self.workspace);
        let url = self.base.join(&path)?;

        <HttpClient as JsonClient>::post(&self.http, self, url, request).await
    }
}
