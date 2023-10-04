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

use card_helpers::{abilities, costs, text, this};
use game_data::card_definition::{AttackBoost, CardConfigBuilder, CardDefinition};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::primitives::{CardSubtype, CardType, Rarity, Resonance, School, Side};
use game_data::special_effects::{Projectile, ProjectileData};
use game_data::text::TextToken::*;

pub fn pathfinder(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::Pathfinder,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(3),
        image: assets::champion_card(meta, "pathfinder"),
        card_type: CardType::Artifact,
        subtypes: vec![CardSubtype::Weapon],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::standard(
                text![Plus(meta.upgrade(2, 4)), "attack in", OuterRooms],
                this::base_attack(|g, s, _, current| {
                    let added = g
                        .raid
                        .as_ref()
                        .and_then(|r| r.target.is_outer_room().then_some(s.upgrade(2, 4)))
                        .unwrap_or(0);
                    current + added
                }),
            ),
            abilities::encounter_boost(),
        ],
        config: CardConfigBuilder::new()
            .base_attack(1)
            .attack_boost(AttackBoost { cost: 1, bonus: 1 })
            .resonance(Resonance::Infernal)
            .combat_projectile(ProjectileData::new(Projectile::Projectiles1(4)))
            .build(),
    }
}
