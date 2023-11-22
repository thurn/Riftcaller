// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use card_helpers::costs;
use core_data::game_primitives::{CardType, Rarity, School, Side};
use game_data::card_definition::{CardConfigBuilder, CardDefinition};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;

pub fn zain_cunning_diplomat(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ZainCuningDiplomat,
        sets: vec![CardSetName::Beryl],
        cost: costs::riftcaller(),
        image: assets::champion_card(meta, "zain"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Riftcaller,
        abilities: vec![],
        config: CardConfigBuilder::new().build(),
    }
}
