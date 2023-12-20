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

use card_helpers::{costs, history, in_play, show_prompt, text, this};
use core_data::adventure_primitives::{Coins, Skill};
use core_data::game_primitives::{
    CardType, GameObjectId, InitiatedBy, Rarity, RoomId, School, Side,
};
use core_ui::design::{self, TimedEffectDataExt};
use game_data::card_definition::{Ability, CardConfigBuilder, CardDefinition, IdentityConfig};
use game_data::card_name::{CardMetadata, CardName};
use game_data::card_set_name::CardSetName;
use game_data::card_state::CardPosition;
use game_data::game_actions::ButtonPromptContext;
use game_data::game_effect::GameEffect;
use game_data::game_state::GameState;
use game_data::prompt_data::{FromZone, PromptChoice, PromptData};
use game_data::random;
use game_data::special_effects::{SoundEffect, TimedEffect, TimedEffectData};
use game_data::text::TextToken::*;
use rules::visual_effects::{ShowAlert, VisualEffects};
use rules::{
    curses, custom_state, draw_cards, end_raid, mana, mutations, prompts, visual_effects,
    CardDefinitionExt,
};

pub fn illea_the_high_sage(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::IlleaTheHighSage,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::riftcaller_card(meta, "Riptaid/illea"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Identity,
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
            .identity(IdentityConfig {
                starting_coins: Coins(500),
                secondary_schools: vec![School::Law],
                skills: vec![Skill::Lore, Skill::Persuasion],
                bio: "Illea's wisdom was nurtured in the ancient libraries of Elandor, where the \
                whispers of the past and future converge. A guardian of knowledge, her mind is a \
                living archive of the ages, every word a thread in the tapestry of history.",
            })
            .build(),
    }
}

pub fn strazihar_the_all_seeing(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::StraziharTheAllSeeing,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::riftcaller_card(meta, "Riptaid/strazihar"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Identity,
        abilities: vec![Ability::new_with_delegate(
            text![
                "The first time the Covenant plays a",
                Permanent,
                "each turn, reveal that card unless they",
                PayMana(1)
            ],
            in_play::on_card_played(|g, s, played| {
                if played.card_id.side == Side::Covenant
                    && g.card(played.card_id).definition().is_permanent()
                {
                    custom_state::identity_once_per_turn(g, s, |g, s| {
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
                        prompts::push_with_data(
                            g,
                            Side::Covenant,
                            s,
                            PromptData::Card(played.card_id),
                        );

                        Ok(())
                    })?;
                }

                Ok(())
            }),
        )
        .delegate(this::prompt(|_, s, source, _| {
            let PromptData::Card(card_id) = source.data else {
                return None;
            };
            show_prompt::with_context_and_choices(
                ButtonPromptContext::PayToPreventRevealing(1),
                vec![
                    PromptChoice::new().effect(GameEffect::ManaCost(
                        Side::Covenant,
                        1,
                        s.initiated_by(),
                    )),
                    PromptChoice::new().effect(GameEffect::RevealCard(card_id)),
                ],
            )
        }))],
        config: CardConfigBuilder::new()
            .identity(IdentityConfig {
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

pub fn godmir_spark_of_defiance(meta: CardMetadata) -> CardDefinition {
    fn should_fire(g: &GameState) -> bool {
        history::accessed_this_turn(g, RoomId::Crypt)
            && history::accessed_this_turn(g, RoomId::Sanctum)
    }

    CardDefinition {
        name: CardName::GodmirSparkOfDefiance,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::riftcaller_card(meta, "iobard/godmir"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Identity,
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
            .identity(IdentityConfig {
                starting_coins: Coins(425),
                secondary_schools: vec![School::Beyond],
                skills: vec![Skill::Brawn, Skill::Lore],
                bio: "Godmir, hailed as the Spark of Defiance, emerged from the scorching heart \
                of Khazpar's volcanic realms, a warrior tempered by fire and strife. Wielding an \
                axe that blazed like the molten rivers of his homeland, he became a legend through \
                his fiery raids, his attacks as unpredictable and ferocious as the land that \
                forged him.",
            })
            .build(),
    }
}

pub fn oleus_the_watcher(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::OleusTheWatcher,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::riftcaller_card(meta, "Riptaid/oleus"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Identity,
        abilities: vec![Ability::new_with_delegate(
            text!["The first time each turn the Covenant summons a minion,", GainMana(2)],
            in_play::on_minion_summoned(|g, s, _| {
                custom_state::identity_once_per_turn(g, s, |g, _| {
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

                    mana::gain(g, Side::Riftcaller, 2);
                    Ok(())
                })?;
                Ok(())
            }),
        )],
        config: CardConfigBuilder::new()
            .identity(IdentityConfig {
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

pub fn ellisar_forgekeeper(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EllisarForgekeeper,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::riftcaller_card(meta, "Riptaid/ellisar"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Identity,
        abilities: vec![Ability::new_with_delegate(
            text!["The first time each turn you sacrifice an artifact,", GainActions(1)],
            in_play::on_card_sacrificed(|g, s, card_id| {
                if card_id.side == Side::Riftcaller && g.card(*card_id).definition().is_artifact() {
                    custom_state::identity_once_per_turn(g, s, |g, _| {
                        VisualEffects::new()
                            .ability_alert(s)
                            .timed_effect(
                                GameObjectId::CardId(s.card_id()),
                                TimedEffectData::new(TimedEffect::MagicCircles1(10))
                                    .scale(1.0)
                                    .sound(SoundEffect::LightMagic("RPG3_LightMagic_Buff01"))
                                    .effect_color(design::YELLOW_900),
                            )
                            .apply(g);
                        mutations::gain_action_points(g, Side::Riftcaller, 1)
                    })?;
                }
                Ok(())
            }),
        )],
        config: CardConfigBuilder::new()
            .identity(IdentityConfig {
                starting_coins: Coins(500),
                secondary_schools: vec![School::Pact],
                skills: vec![Skill::Stealth, Skill::Persuasion],
                bio: "Ellisar's tale was forged in the fiery heart of Khazpar, among cascading \
                rivers of molten rock. His hands, once roughened by the relentless forge, now \
                cradle the secrets of creation, guarding the sacred flame that births both weapon \
                and wonder.",
            })
            .build(),
    }
}

pub fn seldanna_regal_pyromancer(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::SeldannaRegalPyromancer,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::riftcaller_card(meta, "seldanna"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Beyond,
        rarity: Rarity::Identity,
        abilities: vec![Ability::new(text![
            "When you use a",
            RazeAbility,
            "while accessing the",
            Vault,
            ", discard another random card from the",
            Vault
        ])
        .delegate(in_play::on_card_razed(|g, s, event| {
            if event.target() == RoomId::Vault {
                if let Some(card_id) = random::card_in_position(
                    g,
                    Side::Covenant,
                    CardPosition::DeckUnknown(Side::Covenant),
                ) {
                    visual_effects::show(g, s, s.card_id(), ShowAlert::Yes);
                    mutations::discard_card(g, card_id)?;
                }
            }
            Ok(())
        }))],
        config: CardConfigBuilder::new()
            .identity(IdentityConfig {
                starting_coins: Coins(450),
                secondary_schools: vec![School::Shadow],
                skills: vec![Skill::Stealth, Skill::Persuasion],
                bio: "Seldanna's flames were kindled in the fiery depths of Khazpar's Cinderpeak \
                Mountain. A mistress of fire, her presence is as mesmerizing and fierce as the \
                blaze she commands, her touch igniting the very air with regal majesty.",
            })
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles1(1))
                    .scale(2.0)
                    .sound(SoundEffect::WaterMagic("RPG3_WaterMagic2_LightImpact03"))
                    .effect_color(design::BLUE_500),
            )
            .build(),
    }
}

pub fn rolant_the_restorer(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::RolantTheRestorer,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::riftcaller_card(meta, "Kalleeck/rolant"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Identity,
        abilities: vec![Ability::new(text![
            "When you play a",
            Permanent,
            "from your discard pile, draw a card"
        ])
        .delegate(in_play::on_card_played(|g, s, play| {
            if play.from_zone == FromZone::Discard
                && play.card_id.side == Side::Riftcaller
                && g.card(play.card_id).definition().is_permanent()
            {
                visual_effects::show(g, s, s.card_id(), ShowAlert::Yes);
                draw_cards::run(g, Side::Riftcaller, 1, s.initiated_by())?;
            }
            Ok(())
        }))],
        config: CardConfigBuilder::new()
            .identity(IdentityConfig {
                starting_coins: Coins(550),
                secondary_schools: vec![School::Shadow],
                skills: vec![Skill::Brawn, Skill::Persuasion],
                bio: "In the heart of Khazpar's Magma Caverns, Rolant found his calling amidst \
                smoldering ashes. From the cinders of destruction, he learned to resurrect and \
                renew, wielding the fiery breath of creation with an artificer’s gentle touch.",
            })
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles1(3))
                    .scale(1.5)
                    .sound(SoundEffect::LightMagic("RPG3_LightMagic_Buff02"))
                    .effect_color(design::YELLOW_900),
            )
            .build(),
    }
}

pub fn eria_the_ghost_of_vasilor(meta: CardMetadata) -> CardDefinition {
    CardDefinition {
        name: CardName::EriaTheGhostOfVasilor,
        sets: vec![CardSetName::Beryl],
        cost: costs::identity(),
        image: assets::riftcaller_card(meta, "iobard/eria"),
        card_type: CardType::Riftcaller,
        subtypes: vec![],
        side: Side::Riftcaller,
        school: School::Law,
        rarity: Rarity::Identity,
        abilities: vec![Ability::new(text![
            "The first time each turn a Covenant ability would end the raid,",
            "prevent that ability and take a",
            Curse
        ])
        .delegate(in_play::on_ability_will_end_raid(|g, s, event| {
            if event.data.side() == Side::Covenant {
                custom_state::identity_once_per_turn(g, s, |g, s| {
                    visual_effects::show(g, s, s.card_id(), ShowAlert::Yes);
                    end_raid::prevent(g);
                    curses::give_curses(g, s, 1)
                })?;
            }
            Ok(())
        }))],
        config: CardConfigBuilder::new()
            .identity(IdentityConfig {
                starting_coins: Coins(400),
                secondary_schools: vec![School::Pact],
                skills: vec![Skill::Stealth, Skill::Lore],
                bio: "Eria’s legend echoes through the abandoned streets of Vasilor, a haunting \
                melody of loss and vengeance. Emerging from the shadows of betrayal, she moves \
                like a wraith, her vengeance a cold wind sweeping through the forsaken ruins.",
            })
            .visual_effect(
                TimedEffectData::new(TimedEffect::MagicCircles1(6))
                    .scale(1.5)
                    .sound(SoundEffect::LightMagic("RPG3_LightMagic_Cast01"))
                    .effect_color(design::YELLOW_900),
            )
            .build(),
    }
}
