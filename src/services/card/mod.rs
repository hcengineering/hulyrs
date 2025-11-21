use crate::services::card;
use crate::services::core::classes::{Blobs, MarkupBlobRef, Ref};
use crate::services::core::classes::{Rank, UXObject};
use crate::services::event::{Class, DocT, HasId};
use crate::services::preference::Preference;
use crate::services::transactor::tx::Doc;
use crate::services::ui::IconProps;
use serde::{Deserialize, Serialize};

#[allow(non_upper_case_globals)]
pub mod class {
    pub const CardSpace: &str = "card:class:CardSpace";
    pub const Card: &str = "card:class:Card";
    pub const MasterTag: &str = "card:class:MasterTag";
    pub const FavoriteCard: &str = "card:class:FavoriteCard";
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MasterTag {
    #[serde(flatten)]
    pub doc: Doc,
    #[serde(flatten)]
    pub ux_obj: UXObject,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

impl DocT for MasterTag {
    fn doc(&self) -> &Doc {
        &self.doc
    }
}

impl Class for MasterTag {
    const CLASS: &'static str = class::MasterTag;
}

impl HasId for MasterTag {
    fn id(&self) -> &str {
        &self.doc().id
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParentInfo {
    pub _id: Ref,
    pub _class: Ref,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    #[serde(flatten)]
    pub doc: Doc,
    #[serde(flatten)]
    pub icon_props: IconProps,
    pub title: String,
    pub content: MarkupBlobRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blobs: Option<Blobs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_info: Option<Vec<ParentInfo>>,
    pub parent: Option<Box<Card>>,
    pub rank: Option<Rank>,
}

impl DocT for Card {
    fn doc(&self) -> &Doc {
        &self.doc
    }
}

impl Class for Card {
    const CLASS: &'static str = card::class::Card;
}

impl HasId for Card {
    fn id(&self) -> &str {
        &self.doc().id
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteCard {
    #[serde(flatten)]
    pub base: Preference,
    pub attached_to: Ref,
    pub application: String,
}

impl DocT for FavoriteCard {
    fn doc(&self) -> &Doc {
        &self.base.doc
    }
}

impl Class for FavoriteCard {
    const CLASS: &'static str = card::class::FavoriteCard;
}

impl HasId for FavoriteCard {
    fn id(&self) -> &str {
        &self.doc().id
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct CardSpace;

impl Class for CardSpace {
    const CLASS: &'static str = card::class::CardSpace;
}
