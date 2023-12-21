// Copyright © Riftcaller 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use adventure_data::adventure_effect_data::{AdventureEffectKind, DeckCardAction};
use adventure_data::card_filter_data::CardFilterCategoryOperator;
use core_data::adventure_primitives::Skill;
use game_data::card_name::CardName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum NarrativeEventEntryKind {
    Introduction,
    Choice,
    Outcome,
    Reward,
    Cost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEventRow {
    #[serde(rename = "ID")]
    pub id: u32,
    #[serde(rename = "Image Path")]
    pub image_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEventDetailsRow {
    #[serde(rename = "ID")]
    pub id: u32,
    #[serde(rename = "Entry Kind")]
    pub entry_kind: NarrativeEventEntryKind,
    #[serde(rename = "Choice ID")]
    pub choice_id: Option<u32>,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Required Skill")]
    pub required_skill: Option<Skill>,
    #[serde(rename = "Effect Kind")]
    pub effect_kind: Option<AdventureEffectKind>,
    #[serde(rename = "Quantity")]
    pub quantity: Option<u32>,
    #[serde(rename = "Card Filter ID")]
    pub card_filter_id: Option<u32>,
    #[serde(rename = "Deck Card Action")]
    pub deck_card_action: Option<DeckCardAction>,
    #[serde(rename = "Card Name")]
    pub card_name: Option<CardName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardFilterRow {
    #[serde(rename = "ID")]
    pub id: u32,
    #[serde(rename = "Category Operator")]
    pub category_operator: CardFilterCategoryOperator,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Upgraded: No")]
    pub upgraded_no: bool,
    #[serde(rename = "Upgraded: Yes")]
    pub upgraded_yes: bool,
    #[serde(rename = "Rarity: Common")]
    pub rarity_common: bool,
    #[serde(rename = "Rarity: Uncommon")]
    pub rarity_uncommon: bool,
    #[serde(rename = "Rarity: Rare")]
    pub rarity_rare: bool,
    #[serde(rename = "Rarity: Basic")]
    pub rarity_basic: bool,
    #[serde(rename = "Rarity: Identity")]
    pub rarity_identity: bool,
    #[serde(rename = "Rarity: None")]
    pub rarity_none: bool,
    #[serde(rename = "Type: Riftcaller")]
    pub type_riftcaller: bool,
    #[serde(rename = "Type: Chapter")]
    pub type_chapter: bool,
    #[serde(rename = "Type: Game Modifier")]
    pub type_game_modifier: bool,
    #[serde(rename = "Type: Sigil")]
    pub type_sigil: bool,
    #[serde(rename = "Type: Scheme")]
    pub type_scheme: bool,
    #[serde(rename = "Type: Spell")]
    pub type_spell: bool,
    #[serde(rename = "Type: Ritual")]
    pub type_ritual: bool,
    #[serde(rename = "Type: Evocation")]
    pub type_evocation: bool,
    #[serde(rename = "Type: Ally")]
    pub type_ally: bool,
    #[serde(rename = "Type: Project")]
    pub type_project: bool,
    #[serde(rename = "Type: Artifact")]
    pub type_artifact: bool,
    #[serde(rename = "Type: Minion")]
    pub type_minion: bool,
    #[serde(rename = "Side: Covenant")]
    pub side_covenant: bool,
    #[serde(rename = "Side: Riftcaller")]
    pub side_riftcaller: bool,
    #[serde(rename = "Subtype: Weapon")]
    pub subtype_weapon: bool,
}
