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

use std::cmp;

use card_helpers::effects::Effects;
use card_helpers::{costs, history, in_play, text};
use core_data::game_primitives::{CardSubtype, CardType, GameObjectId, Rarity, School, Side};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::card_definition::{Ability, CardConfigBuilder, CardDefinition};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::special_effects::{SoundEffect, TimedEffect, TimedEffectData};
use with_error::fail;

pub fn magistrates_thronehall(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::MagistratesThronehall,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(meta.upgrade(1, 0)),
        image: assets::overlord_card(meta, "magistrates_thronehall"),
        card_type: CardType::Project,
        subtypes: vec![CardSubtype::Nightbound, CardSubtype::Dictate],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Rare,
        abilities: vec![Ability::new_with_delegate(
            text!["The Champion cannot draw more than", 2, "cards during their turn"],
            in_play::on_will_draw_cards(|g, s, _| {
                let drawn_this_turn = history::counters(g, Side::Champion).cards_drawn;
                let mut show_vfx = false;
                let Some(state) = g.state_machines.draw_cards.last_mut() else {
                    fail!("Expected active draw_cards state machine");
                };
                if state.side == Side::Champion {
                    let new = cmp::min(2u32.saturating_sub(drawn_this_turn), state.quantity);
                    if new < state.quantity {
                        show_vfx = true;
                    }
                    state.quantity = new;
                }

                if show_vfx {
                    Effects::new()
                        .ability_alert(s)
                        .timed_effect(
                            GameObjectId::CardId(s.card_id()),
                            TimedEffectData::new(TimedEffect::MagicCircles1(6))
                                .scale(2.0)
                                .sound(SoundEffect::LightMagic("RPG3_LightMagic_Buff01"))
                                .effect_color(design::YELLOW_900),
                        )
                        .apply(g);
                }
                Ok(())
            }),
        )],
        config: CardConfigBuilder::new().raze_cost(5).build(),
    }
}
