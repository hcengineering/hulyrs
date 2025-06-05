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

use crate::services::transactor::document::Doc;
use crate::services::types::AccountUuid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Space {
    #[serde(flatten)]
    pub doc: Doc,
    pub name: String,
    pub private: bool,
    pub members: Vec<AccountUuid>,
    pub archived: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owners: Option<Vec<AccountUuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_join: Option<bool>,
}
