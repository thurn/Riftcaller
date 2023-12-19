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
use card_helpers::play_card_browser_builder::PlayCardBrowserBuilder;
use card_helpers::{costs, history, in_play, show_prompt, text, text_helpers, this};
use core_data::adventure_primitives::{Coins, Skill};
use core_data::game_primitives::{CardType, GameObjectId, Rarity, School, Side};
use core_ui::design::{self, TimedEffectDataExt};
use game_data::card_definition::{
    Ability, ActivatedAbility, CardConfigBuilder, CardDefinition, IdentityConfig,
};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::{CardCounter, CardIdsExt};
use game_data::custom_card_state::CustomCardState;
use game_data::game_actions::ButtonPromptContext;
use game_data::game_effect::GameEffect;
use game_data::prompt_data::{
    BrowserPromptTarget, BrowserPromptValidation, FromZone, PromptChoice, PromptChoiceLabel,
    PromptContext, PromptData,
};
use game_data::special_effects::{SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextToken::*;
use rules::mutations::RealizeCards;
use rules::visual_effects::{ShowAlert, VisualEffects};
use rules::{
    curses, custom_state, flags, mana, mutations, prompts, visual_effects, CardDefinitionExt,
};

pub fn nimbus_enclave(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::NimbusEnclave,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::chapter(meta, "SceneryClouds_1"),
        card_type: CardType::Chapter,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Identity,
        abilities: vec![Ability::new_with_delegate(
            text![
                "When the Riftcaller spends or loses mana during a raid due to a Covenant ability,",
                GainMana(1)
            ],
            in_play::on_mana_lost_to_opponent_ability(|g, s, lost| {
                if lost.side == Side::Riftcaller && flags::raid_active(g) {
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
            .identity(IdentityConfig {
                starting_coins: Coins(500),
                secondary_schools: vec![School::Shadow],
                skills: vec![Skill::Lore, Skill::Persuasion],
                bio: "High above the clouds of Khazpar, the Nimbus Enclave manipulates the stormy \
                heavens. In their floating citadels, they conjure tempests and gales, bending the \
                sky's fury to their inscrutable will.",
            })
            .build(),
    }
}

pub fn enforcers_of_silence(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EnforcersOfSilence,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::chapter(meta, "SceneryClouds_2"),
        card_type: CardType::Chapter,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Law,
        rarity: Rarity::Identity,
        abilities: vec![Ability::new(text![
            "The Riftcaller cannot score more than one scheme per turn"
        ])
        .delegate(in_play::can_score_accessed_card(|g, _, _, current| {
            current.add_constraint(history::counters(g, Side::Riftcaller).schemes_scored == 0)
        }))
        .delegate(in_play::on_will_populate_access_prompt(|g, s, _| {
            if history::counters(g, Side::Riftcaller).schemes_scored > 0 {
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
            .identity(IdentityConfig {
                starting_coins: Coins(475),
                secondary_schools: vec![School::Beyond],
                skills: vec![Skill::Brawn, Skill::Lore],
                bio: "Born in the hushed cloisters of Elandor's forgotten monasteries, the \
                Enforcers of Silence swore oaths to extinguish the world's clamor. Their magic, \
                a silent shroud, smothers voices and secrets alike.",
            })
            .build(),
    }
}

pub fn keepers_of_the_eye(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::KeepersOfTheEye,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::chapter(meta, "SceneryLighthouse_inside_1"),
        card_type: CardType::Chapter,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Beyond,
        rarity: Rarity::Identity,
        abilities: vec![Ability::new_with_delegate(
            text![
                "The first time each turn the Riftcaller loses or spends",
                ActionSymbol,
                "during a raid, you may add a card from the",
                Crypt,
                "to the top of the",
                Vault
            ],
            in_play::on_action_points_lost_during_raid(|g, s, side| {
                if *side == Side::Riftcaller {
                    custom_state::identity_once_per_turn(g, s, |g, s| {
                        VisualEffects::new()
                            .timed_effect(
                                s.card_id(),
                                TimedEffectData::new(TimedEffect::MagicCircles1(5))
                                    .scale(2.0)
                                    .sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Cast01"))
                                    .effect_color(design::BLUE_500),
                            )
                            .ability_alert(s)
                            .apply(g);
                        prompts::push(g, Side::Covenant, s);
                        Ok(())
                    })?;
                }
                Ok(())
            }),
        )
        .delegate(this::prompt(|g, s, _, _| {
            CardSelectorPromptBuilder::new(s, BrowserPromptTarget::Deck)
                .subjects(g.discard_pile(Side::Covenant).card_ids())
                .context(PromptContext::MoveToTopOfVault)
                .validation(BrowserPromptValidation::LessThanOrEqualTo(1))
                .build()
        }))],
        config: CardConfigBuilder::new()
            .identity(IdentityConfig {
                starting_coins: Coins(525),
                secondary_schools: vec![School::Law],
                skills: vec![Skill::Stealth, Skill::Lore],
                bio: "Veiled in the shadows of the Thylen Dominion, the Keepers of the Eye are \
                the silent sentinels of the Mirage Labyrinths. Their vision, steeped in forbidden \
                knowledge, unveils destinies whispered by timeless sands.",
            })
            .build(),
    }
}

pub fn the_starseers(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TheStarseers,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::chapter(meta, "SceneryFrozen_inside_2"),
        card_type: CardType::Chapter,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Beyond,
        rarity: Rarity::Identity,
        abilities: vec![
            Ability::new_with_delegate(
                text_helpers::named_trigger(Dawn, text![text!["Choose a card type"],]),
                in_play::at_dawn(|g, s, _| {
                    prompts::push(g, Side::Covenant, s);
                    Ok(())
                }),
            )
            .delegate(this::prompt(|g, s, _, _| {
                let types =
                    vec![CardType::Artifact, CardType::Evocation, CardType::Ally, CardType::Spell];
                show_prompt::with_context_and_choices(
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
                )
            })),
            Ability::new_with_delegate(
                text![
                    "The first time the Riftcaller plays a card of the chosen type each turn,",
                    GainMana(2),
                ],
                in_play::on_card_played(|g, s, played| {
                    let Some(chosen) =
                        g.card(s.card_id()).custom_state.card_type_for_turn(g.info.turn)
                    else {
                        return Ok(());
                    };

                    if g.card(played.card_id).definition().card_type == chosen {
                        custom_state::identity_once_per_turn(g, s, |g, s| {
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

                            mana::gain(g, Side::Covenant, 2);
                            Ok(())
                        })?;
                    }
                    Ok(())
                }),
            ),
        ],
        config: CardConfigBuilder::new()
            .identity(IdentityConfig {
                starting_coins: Coins(400),
                secondary_schools: vec![School::Shadow],
                skills: vec![Skill::Lore, Skill::Persuasion],
                bio: "Under Frostreach's starlit canopy, the Starseers commune with celestial \
                whispers. Through the dance of snowflakes, they uncover cosmic secrets murmured \
                by the icy heavens.",
            })
            .build(),
    }
}

pub fn rivers_eye(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::RiversEye,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::chapter(meta, "SceneryWater_1"),
        card_type: CardType::Chapter,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Beyond,
        rarity: Rarity::Identity,
        abilities: vec![Ability::new(text![
            "While the Riftcaller is",
            Cursed,
            ", the cards in their hand become visible"
        ])
        .delegate(in_play::on_curse(|g, s, _| {
            visual_effects::show(g, s, s.card_id(), ShowAlert::Yes);
            mutations::set_cards_visible_to(
                g,
                &g.hand(Side::Riftcaller).card_ids(),
                Side::Covenant,
                true,
            );
            Ok(())
        }))
        .delegate(in_play::on_enter_hand(|g, _, &card_id| {
            if card_id.side == Side::Riftcaller && curses::is_riftcaller_cursed(g) {
                mutations::set_visible_to(g, card_id, Side::Covenant, true);
            }
            Ok(())
        }))],
        config: CardConfigBuilder::new()
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles1(4))
                    .scale(1.5)
                    .effect_color(design::BLUE_500)
                    .sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Cast01")),
            )
            .identity(IdentityConfig {
                starting_coins: Coins(475),
                secondary_schools: vec![School::Primal],
                skills: vec![Skill::Stealth, Skill::Persuasion],
                bio: "In Seba's lush heart, the River's Eye thrives, unseen and unpredictable. \
                They weave illusions and realities in the watery depths where secrets flow \
                endlessly.",
            })
            .build(),
    }
}

pub fn the_conjurers_circle(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::TheConjurersCircle,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::chapter(meta, "ScenerySnowMountain_2"),
        card_type: CardType::Chapter,
        subtypes: vec![],
        side: Side::Covenant,
        school: School::Beyond,
        rarity: Rarity::Identity,
        abilities: vec![
            Ability::new(text![
                "The first time each turn the Riftcaller scores a card or uses a",
                RazeAbility,
                ",",
                AddPowerCharges(1)
            ])
            .delegate(in_play::on_riftcaller_scored_card(|g, s, _| {
                custom_state::identity_once_per_turn(g, s, |g, s| {
                    visual_effects::show(g, s, s.card_id(), ShowAlert::Yes);
                    g.card_mut(s).add_counters(CardCounter::PowerCharges, 1);
                    Ok(())
                })
            }))
            .delegate(in_play::on_card_razed(|g, s, _| {
                custom_state::identity_once_per_turn(g, s, |g, s| {
                    visual_effects::show(g, s, s.card_id(), ShowAlert::Yes);
                    g.card_mut(s).add_counters(CardCounter::PowerCharges, 1);
                    Ok(())
                })
            })),
            ActivatedAbility::new(
                costs::power_charges_and_action::<1>(),
                text![
                    text!["Look at the top", 3, "cards of the", Vault],
                    text!["You may play one of them"]
                ],
            )
            .delegate(this::on_activated(|g, s, _| {
                let cards = mutations::realize_top_of_deck(
                    g,
                    Side::Covenant,
                    3,
                    RealizeCards::SetVisibleToOwner,
                )?;

                prompts::push_with_data(g, Side::Covenant, s, PromptData::Cards(cards));
                Ok(())
            }))
            .delegate(this::prompt(|_, s, source, _| {
                let PromptData::Cards(cards) = &source.data else { return None };
                PlayCardBrowserBuilder::new(s, FromZone::Deck, cards.clone()).build()
            }))
            .build(),
        ],
        config: CardConfigBuilder::new()
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles1(5))
                    .scale(1.5)
                    .effect_color(design::BLUE_500)
                    .sound(SoundEffect::WaterMagic("RPG3_WaterMagic_Cast02")),
            )
            .identity(IdentityConfig {
                starting_coins: Coins(500),
                secondary_schools: vec![School::Primal],
                skills: vec![Skill::Stealth, Skill::Persuasion],
                bio: "In the silent snowfall of Frostreach, the Conjurer’s Circle summons \
                moonlit spirits. Their magic conjures ephemeral wonders, disappearing at dawn.",
            })
            .build(),
    }
}
