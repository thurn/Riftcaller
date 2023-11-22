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

use core_data::adventure_primitives::{Coins, Skill};
use core_data::game_primitives::{CardType, GameObjectId, Rarity, School, Side};
use game_data::card_definition::{Ability, CardConfigBuilder, CardDefinition, RiftcallerConfig};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::special_effects::{SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextToken::*;

use card_helpers::{costs, in_play, text};
use card_helpers::effects::Effects;
use core_ui::design::{self, TimedEffectDataExt};
use rules::{flags, mana};

pub fn zain_cunning_diplomat(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ZainCunningDiplomat,
        sets: vec![CardSetName::Beryl],
        cost: costs::riftcaller(),
        image: assets::overlord_card(meta, "zain"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Riftcaller,
        abilities: vec![Ability::new_with_delegate(
            text![
                "When the Champion spends or loses mana during a raid due to an Overlord ability,",
                GainMana(1)
            ],
            in_play::on_mana_lost_to_opponent_ability(|g, s, lost| {
                if lost.side == Side::Champion && flags::raid_active(g) {
                    Effects::new()
                        .ability_alert(s)
                        .timed_effect(
                            GameObjectId::CardId(s.card_id()),
                            TimedEffectData::new(TimedEffect::MagicCircles1(4))
                                .scale(1.0)
                                .sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Buff01"))
                                .effect_color(design::BLUE_500),
                        )
                        .apply(g);

                    mana::gain(g, s.side(), 1);
                }

                Ok(())
            }),
        )],
        config: CardConfigBuilder::new()
            .riftcaller(RiftcallerConfig {
                starting_coins: Coins(500),
                secondary_schools: vec![School::Shadow],
                skills: vec![Skill::Lore, Skill::Persuasion],
            })
            .build(),
    }
}
