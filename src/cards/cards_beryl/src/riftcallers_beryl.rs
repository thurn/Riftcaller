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

use card_helpers::effects::Effects;
use card_helpers::{costs, in_play, text};
use core_data::adventure_primitives::{Coins, Skill};
use core_data::game_primitives::{CardType, GameObjectId, InitiatedBy, Rarity, School, Side};
use core_ui::design::{self, TimedEffectDataExt};
use game_data::card_definition::{Ability, CardConfigBuilder, CardDefinition, RiftcallerConfig};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::special_effects::{SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextToken::*;
use rules::{draw_cards, flags, mana};

// ========================================== //
// ========== Overlord Riftcallers ========== //
// ========================================== //

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
                            TimedEffectData::new(TimedEffect::MagicCircles1(5))
                                .scale(1.0)
                                .sound(SoundEffect::LightMagic("RPG3_LightMagic_Buff01"))
                                .effect_color(design::YELLOW_900),
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
                bio: "Born in the gilded courts of Elandor, Zain's silver tongue and sharp mind \
                mask a heart marred by treachery. His diplomacy is a chess game, each move \
                calculated to weave a web of deceit, turning allies into pawns in his quest for \
                power.",
            })
            .build(),
    }
}

// ========================================== //
// ========== Champion Riftcallers ========== //
// ========================================== //

pub fn illeas_the_high_sage(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::IlleasTheHighSage,
        sets: vec![CardSetName::Beryl],
        cost: costs::riftcaller(),
        image: assets::champion_card(meta, "illeas"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Riftcaller,
        abilities: vec![Ability::new_with_delegate(
            text![
                "The first time each turn you draw cards through a card ability,",
                "draw an additional card"
            ],
            in_play::on_draw_cards_via_ability(|g, s, side| {
                if s.side() == *side
                    && g.current_history_counters(s.side()).cards_drawn_via_abilities == 0
                {
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

                    // Must use SilentAbility to prevent infinite loop
                    draw_cards::run(g, s.side(), 1, InitiatedBy::SilentAbility(s.ability_id()))?;
                }

                Ok(())
            }),
        )],
        config: CardConfigBuilder::new()
            .riftcaller(RiftcallerConfig {
                starting_coins: Coins(500),
                secondary_schools: vec![School::Law],
                skills: vec![Skill::Lore, Skill::Persuasion],
                bio: "Illeas's wisdom was nurtured in the ancient libraries of Elandor, where the \
                whispers of the past and future converge. A guardian of knowledge, his mind is a \
                living archive of the ages, every word a thread in the tapestry of history.",
            })
            .build(),
    }
}
