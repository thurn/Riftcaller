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

use assets::EnvironmentType;
use card_helpers::{history, *};
use game_data::card_definition::{CardConfig, CardDefinition};
use game_data::card_name::CardName;
use game_data::card_set_name::CardSetName;
use game_data::primitives::{CardType, Rarity, School, Side};
use rules::{mana, mutations};

pub fn ennera_imris_blood_bound() -> CardDefinition {
    CardDefinition {
        name: CardName::EnneraImrisBloodBound,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(0),
        image: assets::fantasy_class_image("Warrior", "Female"),
        card_type: CardType::Leader,
        side: Side::Champion,
        school: School::Pact,
        rarity: Rarity::Exalted,
        abilities: vec![simple_ability(
            trigger_text(Dawn, text![Gain, Mana(1), "if you have", 2, "or fewer cards in hand"]),
            in_play::at_dawn(|g, s, _| {
                if g.hand(s.side()).count() <= 2 {
                    alert(g, s);
                    mana::gain(g, s.side(), 1);
                }
                Ok(())
            }),
        )],
        config: CardConfig {
            player_portrait: Some(assets::fantasy_class_portrait(Side::Champion, "Warrior_F")),
            image_background: Some(assets::environments(
                EnvironmentType::CastlesTowersKeeps,
                "Tavern/SceneryTavern_outside_1",
            )),
            ..CardConfig::default()
        },
    }
}

pub fn aris_fey_the_radiant_sun() -> CardDefinition {
    CardDefinition {
        name: CardName::ArisFeyTheRadiantSun,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(3),
        image: assets::fantasy_class_image("Priest", "Female"),
        card_type: CardType::Leader,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Exalted,
        abilities: vec![simple_ability(
            text!["The first time you take damage each turn, draw a card"],
            in_play::on_damage(|g, s, _| {
                once_per_turn(g, s, &(), |g, s, _| {
                    alert(g, s);
                    mutations::draw_cards(g, s.side(), 1)?;
                    Ok(())
                })
            }),
        )],
        config: CardConfig {
            player_portrait: Some(assets::fantasy_class_portrait(Side::Champion, "Priest_F")),
            image_background: Some(assets::environments(
                EnvironmentType::CastlesTowersKeeps,
                "Enchanted/SceneryEForest_outside_1",
            )),
            ..CardConfig::default()
        },
    }
}

pub fn telantes_dugoth_earthbreaker() -> CardDefinition {
    CardDefinition {
        name: CardName::TelantesDugothEarthbreaker,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(3),
        image: assets::fantasy_class_image("Druid", "Male"),
        card_type: CardType::Leader,
        side: Side::Champion,
        school: School::Primal,
        rarity: Rarity::Exalted,
        abilities: vec![simple_ability(
            text!["After you access the", Sanctum, ", discard the top card of the", Vault],
            in_play::after_sanctum_accessed(|g, s, _| {
                alert(g, s);
                mutations::discard_from_vault(g, 1)
            }),
        )],
        config: CardConfig {
            player_portrait: Some(assets::fantasy_class_portrait(Side::Champion, "Druid_M")),
            image_background: Some(assets::environments(
                EnvironmentType::CastlesTowersKeeps,
                "Enchanted/SceneryEForest_inside_2",
            )),
            ..CardConfig::default()
        },
    }
}

pub fn andvari_est_nights_warden() -> CardDefinition {
    CardDefinition {
        name: CardName::AndvariEstNightsWarden,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(1),
        image: assets::fantasy_class_image("Thief", "Male"),
        card_type: CardType::Leader,
        side: Side::Champion,
        school: School::Shadow,
        rarity: Rarity::Exalted,
        abilities: vec![simple_ability(
            text![
                "When you access the",
                Vault,
                ", access a scheme card in the top",
                5,
                "cards, if present"
            ],
            in_play::vault_access_selected(|g, _, _| {
                let cards = mutations::realize_top_of_deck(g, Side::Overlord, 5)?;
                if let Some(card_id) =
                    cards.into_iter().find(|id| rules::card_definition(g, *id).is_scheme())
                {
                    if !g.raid()?.accessed.contains(&card_id) {
                        g.raid_mut()?.accessed.push(card_id);
                    }
                }
                Ok(())
            }),
        )],
        config: CardConfig {
            player_portrait: Some(assets::fantasy_class_portrait(Side::Champion, "Thief_M")),
            image_background: Some(assets::environments(
                EnvironmentType::CastlesTowersKeeps,
                "Fortress/SceneryFortress_inside_1",
            )),
            ..CardConfig::default()
        },
    }
}

pub fn ubras_efaris_time_shaper() -> CardDefinition {
    CardDefinition {
        name: CardName::UbrasEfarisTimeShaper,
        sets: vec![CardSetName::ProofOfConcept],
        cost: cost(4),
        image: assets::fantasy_class_image("Mage", "Male"),
        card_type: CardType::Leader,
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Exalted,
        abilities: vec![simple_ability(
            text!["The second spell you cast each turn does not cost", ActionSymbol],
            in_play::on_query_action_cost(|g, _, _, actions| {
                let cards = history::cards_played_this_turn(g);
                if cards.filter(|id| rules::card_definition(g, *id).is_spell()).count() == 1 {
                    0
                } else {
                    actions
                }
            }),
        )],
        config: CardConfig {
            player_portrait: Some(assets::fantasy_class_portrait(Side::Champion, "Mage_M")),
            image_background: Some(assets::environments(
                EnvironmentType::CastlesTowersKeeps,
                "Forge/SceneryForge_outside_1",
            )),
            ..CardConfig::default()
        },
    }
}
