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

use core_data::game_primitives::{CardType, Rarity, School, Side};
use game_data::card_definition::{Ability, CardConfig, CardDefinition};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::delegate_data::{Delegate, EventDelegate};

use assets::rexard_images;
use card_helpers::*;
use card_helpers::effects::Effects;
use card_helpers::requirements::FaceUpInPlay;
use card_helpers::text_helpers::named_trigger;
use rules::{CardDefinitionExt, mana, mutations};

pub fn ennera_imris(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EnneraImrisBloodBound,
        sets: vec![CardSetName::Amethyst],
        cost: costs::riftcaller(),
        image: rexard_images::spell(8, "SpellBook08_09"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Rare,
        abilities: vec![Ability::new_with_delegate(
            named_trigger(Dawn, text![GainMana(1), "if you have", 2, "or fewer cards in hand"]),
            in_play::at_dawn(|g, s, _| {
                if g.hand(s.side()).count() <= 2 {
                    Effects::new().ability_alert(s).apply(g);
                    mana::gain(g, s.side(), 1);
                }
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn aris_fey(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::ArisFeyTheRadiantSun,
        sets: vec![CardSetName::Amethyst],
        cost: costs::riftcaller(),
        image: rexard_images::spell(8, "SpellBook08_73"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Pact,
        rarity: Rarity::Riftcaller,
        abilities: vec![Ability::new_with_delegate(
            text!["The first time you take damage each turn, draw a card"],
            Delegate::DealtDamage(EventDelegate {
                requirement: requirements::no_damage_dealt::<FaceUpInPlay>,
                mutation: |g, s, _| {
                    Effects::new().ability_alert(s).apply(g);
                    mutations::draw_cards(g, s.side(), 1)?;
                    Ok(())
                },
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn telantes_dugoth(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TelantesDugothEarthbreaker,
        sets: vec![CardSetName::Amethyst],
        cost: costs::riftcaller(),
        image: rexard_images::spell(8, "SpellBook08_76"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Primal,
        rarity: Rarity::Riftcaller,
        abilities: vec![Ability::new_with_delegate(
            text!["After you access the", Sanctum, ", discard the top card of the", Vault],
            in_play::after_sanctum_accessed(|g, s, _| {
                Effects::new().ability_alert(s).apply(g);
                mutations::discard_from_vault(g, 1)
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn andvari_est(_: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::AndvariEstNightsWarden,
        sets: vec![CardSetName::Amethyst],
        cost: costs::riftcaller(),
        image: rexard_images::spell(8, "SpellBook08_119"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Shadow,
        rarity: Rarity::Riftcaller,
        abilities: vec![Ability::new_with_delegate(
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
                    cards.into_iter().find(|id| g.card(*id).definition().is_scheme())
                {
                    if !g.raid()?.accessed.contains(&card_id) {
                        g.raid_mut()?.accessed.push(card_id);
                    }
                }
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}
