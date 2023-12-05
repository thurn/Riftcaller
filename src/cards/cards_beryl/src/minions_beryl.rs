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

use card_helpers::{combat_abilities, costs, delegates, requirements, text, text_helpers, this};
use core_data::game_primitives::{CardSubtype, CardType, GameObjectId, Rarity, School, Side};
use core_ui::design;
use core_ui::design::TimedEffectDataExt;
use game_data::card_definition::{Ability, CardConfigBuilder, CardDefinition, Resonance};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::special_effects::{
    Projectile, ProjectileData, SoundEffect, TimedEffect, TimedEffectData,
};
use game_data::text::TextToken::*;
use rules::visual_effects::VisualEffects;
use with_error::fail;

pub fn incarnation_of_justice(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::IncarnationOfJustice,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(0),
        image: assets::overlord_card(meta, "incarnation_of_justice"),
        card_type: CardType::Minion,
        subtypes: vec![CardSubtype::Fey],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Rare,
        abilities: vec![Ability::new_with_delegate(
            text_helpers::named_trigger(Combat, text!["The Champion cannot draw cards this turn"]),
            delegates::on_will_draw_cards(
                requirements::combat_ability_fired_this_turn,
                |g, s, _| {
                    let Some(state) = g.state_machines.draw_cards.last_mut() else {
                        fail!("Expected active draw_cards state machine");
                    };
                    state.draw_is_prevented = true;

                    VisualEffects::new()
                        .ability_alert(s)
                        .timed_effect(
                            GameObjectId::CardId(s.card_id()),
                            TimedEffectData::new(TimedEffect::MagicCircles1(7))
                                .scale(2.0)
                                .sound(SoundEffect::LightMagic(
                                    "RPG3_LightMagicMisc_AttackMissed04",
                                ))
                                .effect_color(design::YELLOW_900),
                        )
                        .apply(g);
                    Ok(())
                },
            ),
        )],
        config: CardConfigBuilder::new()
            .health(5)
            .shield(meta.upgrade(1, 3))
            .resonance(Resonance::mortal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(4))
                    .fire_sound(SoundEffect::LightMagic("RPG3_LightMagic3_Projectile01"))
                    .impact_sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_Impact01")),
            )
            .build(),
    }
}

pub fn sentinel_sphinx(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SentinelSphinx,
        sets: vec![CardSetName::Beryl],
        cost: costs::mana(4),
        image: assets::overlord_card(meta, "sentinel_sphinx"),
        card_type: CardType::Minion,
        subtypes: vec![CardSubtype::Beast],
        side: Side::Overlord,
        school: School::Beyond,
        rarity: Rarity::Common,
        abilities: vec![
            Ability::new_with_delegate(
                text!["This minion cannot be", Evaded],
                this::can_evade(delegates::disallow),
            ),
            combat_abilities::end_raid(),
        ],
        config: CardConfigBuilder::new()
            .health(meta.upgrade(2, 3))
            .shield(meta.upgrade(1, 2))
            .resonance(Resonance::infernal())
            .combat_projectile(
                ProjectileData::new(Projectile::Projectiles1(6))
                    .fire_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Projectiles01"))
                    .impact_sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Impact01")),
            )
            .build(),
    }
}
