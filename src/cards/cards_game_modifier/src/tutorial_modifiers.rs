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

use assets::rexard_images;
use card_helpers::{text2, *};
use game_data::card_definition::{Ability, CardConfig, CardDefinition, Cost};
use game_data::card_name::CardName;
use game_data::card_set_name::CardSetName;
use game_data::delegates::{Delegate, EventDelegate, QueryDelegate};
use game_data::primitives::{CardType, Rarity, RoomId, School, Side};

fn tutorial_modifier(name: CardName, ability: Ability) -> CardDefinition {
    CardDefinition {
        name,
        sets: vec![CardSetName::TutorialEffects],
        cost: Cost::default(),
        image: rexard_images::spell(1, "SpellBook01_01"),
        card_type: CardType::GameModifier,
        side: Side::Overlord,
        school: School::Neutral,
        rarity: Rarity::Common,
        abilities: vec![ability],
        config: CardConfig::default(),
    }
}

pub fn overlord_empty_modifier() -> CardDefinition {
    tutorial_modifier(CardName::OverlordEmptyModifier, text_only_ability(text2!["No effect"]))
}

pub fn tutorial_disable_draw_action() -> CardDefinition {
    tutorial_modifier(
        CardName::TutorialDisableDrawAction,
        simple_ability(
            text2!["The Champion cannot take the 'draw card' action"],
            Delegate::CanTakeDrawCardAction(QueryDelegate {
                requirement: side_is_champion,
                transformation: |_, _, _, f| f.with_override(false),
            }),
        ),
    )
}

pub fn tutorial_disable_gain_mana() -> CardDefinition {
    tutorial_modifier(
        CardName::TutorialDisableGainMana,
        simple_ability(
            text2!["The Champion cannot take the 'gain mana' action"],
            Delegate::CanTakeGainManaAction(QueryDelegate {
                requirement: side_is_champion,
                transformation: |_, _, _, f| f.with_override(false),
            }),
        ),
    )
}

pub fn tutorial_disable_raid_sanctum() -> CardDefinition {
    tutorial_modifier(
        CardName::TutorialDisableRaidSanctum,
        simple_ability(
            text2!["The Champion cannot raid the Sanctum"],
            Delegate::CanInitiateRaid(QueryDelegate {
                requirement: room_is_sanctum,
                transformation: |_, _, _, f| f.with_override(false),
            }),
        ),
    )
}

pub fn tutorial_disable_raid_vault() -> CardDefinition {
    tutorial_modifier(
        CardName::TutorialDisableRaidVault,
        simple_ability(
            text2!["The Champion cannot raid the Vault"],
            Delegate::CanInitiateRaid(QueryDelegate {
                requirement: room_is_vault,
                transformation: |_, _, _, f| f.with_override(false),
            }),
        ),
    )
}

pub fn tutorial_disable_raid_crypts() -> CardDefinition {
    tutorial_modifier(
        CardName::TutorialDisableRaidCrypts,
        simple_ability(
            text2!["The Champion cannot raid the Crypts"],
            Delegate::CanInitiateRaid(QueryDelegate {
                requirement: room_is_crypts,
                transformation: |_, _, _, f| f.with_override(false),
            }),
        ),
    )
}

pub fn tutorial_disable_raid_outer() -> CardDefinition {
    tutorial_modifier(
        CardName::TutorialDisableRaidOuter,
        simple_ability(
            text2!["The Champion cannot raid outer rooms"],
            Delegate::CanInitiateRaid(QueryDelegate {
                requirement: |_, _, room_id| room_id.is_outer_room(),
                transformation: |_, _, _, f| f.with_override(false),
            }),
        ),
    )
}

pub fn tutorial_disable_raid_continue() -> CardDefinition {
    tutorial_modifier(
        CardName::TutorialDisableRaidContinue,
        simple_ability(
            text2!["The Champion must use a weapon during raids"],
            Delegate::CanUseNoWeapon(QueryDelegate {
                requirement: always,
                transformation: |_, _, _, f| f.with_override(false),
            }),
        ),
    )
}

pub fn tutorial_disable_end_raid() -> CardDefinition {
    tutorial_modifier(
        CardName::TutorialDisableEndRaid,
        simple_ability(
            text2!["The Champion cannot end the access phase of raids"],
            Delegate::CanEndRaidAccessPhase(QueryDelegate {
                requirement: always,
                transformation: |_, _, _, f| f.with_override(false),
            }),
        ),
    )
}

pub fn tutorial_force_sanctum_score() -> CardDefinition {
    tutorial_modifier(
        CardName::TutorialForceSanctumScore,
        simple_ability(
            text2!["The Champion always accesses a scheme card when raiding the Sanctum"],
            Delegate::RaidAccessSelected(EventDelegate {
                requirement: |_, _, event| event.target == RoomId::Sanctum,
                mutation: |g, _, _| {
                    let scheme = g
                        .hand(Side::Overlord)
                        .find(|card| rules::get(card.name).card_type == CardType::Scheme)
                        .map(|c| c.id);
                    if let Some(id) = scheme {
                        g.raid_mut()?.accessed = vec![id];
                    }
                    Ok(())
                },
            }),
        ),
    )
}
