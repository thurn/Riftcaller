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

use card_helpers::card_selector_prompt_builder::CardSelectorPromptBuilder;
use card_helpers::visual_effects::VisualEffects;
use card_helpers::{
    card_selector_prompt_builder, costs, history, in_play, show_prompt, text, text_helpers,
};
use core_data::adventure_primitives::{Coins, Skill};
use core_data::game_primitives::{
    CardType, GameObjectId, InitiatedBy, Rarity, RoomId, School, Side,
};
use core_ui::design::{self, TimedEffectDataExt};
use game_data::card_definition::{Ability, CardConfigBuilder, CardDefinition, RiftcallerConfig};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardIdsExt;
use game_data::custom_card_state::CustomCardState;
use game_data::game_actions::ButtonPromptContext;
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;
use game_data::prompt_data::{
    BrowserPromptTarget, BrowserPromptValidation, PromptChoice, PromptChoiceLabel, PromptContext,
};
use game_data::special_effects::{Projectile, SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextToken::*;
use rules::{custom_state, draw_cards, flags, mana, CardDefinitionExt};

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
                    VisualEffects::new()
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

pub fn algrak_councils_enforcer(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::AlgrakCouncilsEnforcer,
        sets: vec![CardSetName::Beryl],
        cost: costs::riftcaller(),
        image: assets::overlord_card(meta, "algrak"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Riftcaller,
        abilities: vec![Ability::new(text![
            "The Champion cannot score more than one scheme per turn"
        ])
        .delegate(in_play::can_score_accessed_card(|g, _, _, current| {
            current.add_constraint(history::counters(g, Side::Champion).schemes_scored == 0)
        }))
        .delegate(in_play::on_will_populate_access_prompt(|g, s, _| {
            if history::counters(g, Side::Champion).schemes_scored > 0 {
                VisualEffects::new()
                    .ability_alert(s)
                    .timed_effect(
                        GameObjectId::CardId(s.card_id()),
                        TimedEffectData::new(TimedEffect::MagicCircles1(6))
                            .scale(2.0)
                            .sound(SoundEffect::LightMagic("RPG3_LightMagic_Debuff02"))
                            .effect_color(design::YELLOW_900),
                    )
                    .apply(g);
            }
            Ok(())
        }))],
        config: CardConfigBuilder::new()
            .riftcaller(RiftcallerConfig {
                starting_coins: Coins(475),
                secondary_schools: vec![School::Beyond],
                skills: vec![Skill::Brawn, Skill::Lore],
                bio: "From the stern halls of Khazpar, Algrak emerged, an unyielding embodiment \
                of the council's will. His mere presence quells dissent, his iron fist enforcing \
                order with a resolve as relentless as the volcanic land he hails from.",
            })
            .build(),
    }
}

pub fn eria_time_conduit(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EriaTimeConduit,
        sets: vec![CardSetName::Beryl],
        cost: costs::riftcaller(),
        image: assets::overlord_card(meta, "eria"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Beyond,
        rarity: Rarity::Riftcaller,
        abilities: vec![Ability::new_with_delegate(
            text![
                "The first time each turn the Champion loses or spends",
                ActionSymbol,
                "during a raid, you may add a card from the",
                Crypt,
                "to the top of the",
                Vault
            ],
            in_play::on_action_points_lost_during_raid(|g, s, side| {
                if *side == Side::Champion {
                    custom_state::riftcaller_once_per_turn(g, s, |g, s| {
                        card_selector_prompt_builder::show(
                            g,
                            CardSelectorPromptBuilder::new(s, BrowserPromptTarget::Deck)
                                .subjects(g.discard_pile(Side::Overlord).card_ids())
                                .movement_effect(Projectile::Projectiles1(2))
                                .context(PromptContext::MoveToTopOfVault)
                                .show_ability_alert(true)
                                .validation(BrowserPromptValidation::LessThanOrEqualTo(1))
                                .visual_effect(
                                    GameObjectId::CardId(s.card_id()),
                                    TimedEffectData::new(TimedEffect::MagicCircles1(5))
                                        .scale(2.0)
                                        .sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Cast01"))
                                        .effect_color(design::BLUE_500),
                                ),
                        )
                    })?;
                }

                Ok(())
            }),
        )],
        config: CardConfigBuilder::new()
            .riftcaller(RiftcallerConfig {
                starting_coins: Coins(525),
                secondary_schools: vec![School::Law],
                skills: vec![Skill::Stealth, Skill::Lore],
                bio: "Eria’s essence is intertwined with the fleeting threads of time, born under \
                the cosmic alignments of Frostreach. Her journey through the ages is a dance on \
                the edge of now and then, a reluctant guardian of time’s fragile tapestry.",
            })
            .build(),
    }
}

pub fn vendoc_seer_in_starlight(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::VendocSeerInStarlight,
        sets: vec![CardSetName::Beryl],
        cost: costs::riftcaller(),
        image: assets::overlord_card(meta, "vendoc"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Overlord,
        school: School::Beyond,
        rarity: Rarity::Riftcaller,
        abilities: vec![
            Ability::new_with_delegate(
                text_helpers::named_trigger(Dawn, text![text!["Choose a card type"],]),
                in_play::at_dawn(|g, s, _| {
                    let types = vec![
                        CardType::Artifact,
                        CardType::Evocation,
                        CardType::Ally,
                        CardType::Spell,
                    ];
                    show_prompt::with_context_and_choices(
                        g,
                        Side::Overlord,
                        ButtonPromptContext::ChooseCardType(s.card_id()),
                        types
                            .into_iter()
                            .map(|t| {
                                PromptChoice::new()
                                    .effect(GameEffect::AppendCustomCardState(
                                        s.card_id(),
                                        CustomCardState::CardTypeForTurn {
                                            card_type: t,
                                            turn: g.info.turn,
                                        },
                                    ))
                                    .custom_label(PromptChoiceLabel::CardType(t))
                            })
                            .collect(),
                    );
                    Ok(())
                }),
            ),
            Ability::new_with_delegate(
                text![
                    "The first time the Champion plays a card of the chosen type each turn,",
                    GainMana(2),
                ],
                in_play::on_card_played(|g, s, played| {
                    let Some(chosen) =
                        g.card(s.card_id()).custom_state.card_type_for_turn(g.info.turn)
                    else {
                        return Ok(());
                    };

                    if g.card(played.card_id).definition().card_type == chosen {
                        custom_state::riftcaller_once_per_turn(g, s, |g, s| {
                            VisualEffects::new()
                                .ability_alert(s)
                                .timed_effect(
                                    GameObjectId::CardId(s.card_id()),
                                    TimedEffectData::new(TimedEffect::MagicCircles1(8))
                                        .scale(1.0)
                                        .sound(SoundEffect::WaterMagic(
                                            "RPG3_WaterMagic2_HeavyImpact03",
                                        ))
                                        .effect_color(design::BLUE_500),
                                )
                                .apply(g);

                            mana::gain(g, Side::Overlord, 2);
                            Ok(())
                        })?;
                    }
                    Ok(())
                }),
            ),
        ],
        config: CardConfigBuilder::new()
            .riftcaller(RiftcallerConfig {
                starting_coins: Coins(400),
                secondary_schools: vec![School::Shadow],
                skills: vec![Skill::Lore, Skill::Persuasion],
                bio: "Vendoc’s eyes were opened to the celestial secrets in the starlit valleys \
                of Frostreach. His visions, woven from the fabric of starlight, reveal truths \
                that are as much a curse as they are a gift, a seer's burden carried under the \
                cold gaze of the cosmos.",
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
                    VisualEffects::new()
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

pub fn strazihar_the_all_seeing(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::StraziharTheAllSeeing,
        sets: vec![CardSetName::Beryl],
        cost: costs::riftcaller(),
        image: assets::champion_card(meta, "strazihar"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Beyond,
        rarity: Rarity::Riftcaller,
        abilities: vec![Ability::new_with_delegate(
            text![
                "The first time the Overlord plays a",
                Permanent,
                "each turn, reveal that card unless they",
                PayMana(1)
            ],
            in_play::on_card_played(|g, s, played| {
                if played.card_id.side == Side::Overlord
                    && g.card(played.card_id).definition().is_permanent()
                {
                    custom_state::riftcaller_once_per_turn(g, s, |g, s| {
                        VisualEffects::new()
                            .ability_alert(s)
                            .timed_effect(
                                GameObjectId::CardId(s.card_id()),
                                TimedEffectData::new(TimedEffect::MagicCircles1(5))
                                    .scale(2.0)
                                    .sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Buff02"))
                                    .effect_color(design::BLUE_500),
                            )
                            .apply(g);

                        show_prompt::with_context_and_choices(
                            g,
                            Side::Overlord,
                            ButtonPromptContext::PayToPreventRevealing(1),
                            vec![
                                PromptChoice::new().effect(GameEffect::ManaCost(
                                    Side::Overlord,
                                    1,
                                    s.initiated_by(),
                                )),
                                PromptChoice::new().effect(GameEffect::RevealCard(played.card_id)),
                            ],
                        );
                        Ok(())
                    })?;
                }

                Ok(())
            }),
        )],
        config: CardConfigBuilder::new()
            .riftcaller(RiftcallerConfig {
                starting_coins: Coins(400),
                secondary_schools: vec![School::Law],
                skills: vec![Skill::Stealth, Skill::Lore],
                bio: "Born atop the highest peak of Frostreach, Strazihar's eyes were opened to \
                the cosmos during a rare alignment of celestial bodies. Gifted with sight that \
                pierces through realms and time, his gaze unveils the veiled secrets of Ayanor, \
                each blink a revelation.",
            })
            .build(),
    }
}

pub fn merethyl_lore_seeker(meta: CardMetadata) -> CardDefinition {
    fn should_fire(g: &GameState) -> bool {
        history::accessed_this_turn(g, RoomId::Crypt)
            && history::accessed_this_turn(g, RoomId::Sanctum)
    }

    CardDefinition {
        name: CardName::MerethylLoreSeeker,
        sets: vec![CardSetName::Beryl],
        cost: costs::riftcaller(),
        image: assets::champion_card(meta, "merethyl"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Riftcaller,
        abilities: vec![Ability::new_with_delegate(
            text![
                "When you access the",
                Vault,
                ", if you have accessed the",
                Crypt,
                "and",
                Sanctum,
                "this turn, access",
                2,
                "additional cards"
            ],
            in_play::on_query_vault_access_count(|g, _, _, current| {
                if should_fire(g) {
                    current + 2
                } else {
                    current
                }
            }),
        )
        .delegate(in_play::on_vault_access_start(|g, s, _| {
            if should_fire(g) {
                VisualEffects::new()
                    .ability_alert(s)
                    .timed_effect(
                        GameObjectId::CardId(s.card_id()),
                        TimedEffectData::new(TimedEffect::MagicCircles1(8))
                            .scale(1.0)
                            .sound(SoundEffect::LightMagic("RPG3_LightMagic_Buff01"))
                            .effect_color(design::YELLOW_900),
                    )
                    .apply(g);
            }
            Ok(())
        }))],
        config: CardConfigBuilder::new()
            .riftcaller(RiftcallerConfig {
                starting_coins: Coins(425),
                secondary_schools: vec![School::Beyond],
                skills: vec![Skill::Brawn, Skill::Lore],
                bio: "From the verdant depths of the Seban Empire, Merethyl grew up amidst the \
                ancient stones of the Temple of Whispering Vines. Her thirst for ancient knowledge \
                led her through forgotten paths, where the very leaves seemed to murmur arcane \
                secrets into her soul.",
            })
            .build(),
    }
}

pub fn oleus_the_watcher(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::OleusTheWatcher,
        sets: vec![CardSetName::Beryl],
        cost: costs::riftcaller(),
        image: assets::champion_card(meta, "oleus"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Riftcaller,
        abilities: vec![Ability::new_with_delegate(
            text!["The first time each turn the Overlord summons a minion,", GainMana(2)],
            in_play::on_minion_summoned(|g, s, _| {
                custom_state::riftcaller_once_per_turn(g, s, |g, _| {
                    VisualEffects::new()
                        .ability_alert(s)
                        .timed_effect(
                            GameObjectId::CardId(s.card_id()),
                            TimedEffectData::new(TimedEffect::MagicCircles1(9))
                                .scale(1.0)
                                .sound(SoundEffect::LightMagic("RPG3_LightMagic_Buff02"))
                                .effect_color(design::YELLOW_900),
                        )
                        .apply(g);

                    mana::gain(g, Side::Champion, 2);
                    Ok(())
                })?;
                Ok(())
            }),
        )],
        config: CardConfigBuilder::new()
            .riftcaller(RiftcallerConfig {
                starting_coins: Coins(475),
                secondary_schools: vec![School::Beyond],
                skills: vec![Skill::Lore, Skill::Persuasion],
                bio: "Growing up in Elandor's Luminous Glades, Oleus was marked by an ancient \
                prophecy. His gaze, sharper than the keenest blade, has unveiled secrets long \
                buried beneath the whispering breeze of Mystwind Tower.",
            })
            .build(),
    }
}
