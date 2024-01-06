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

//! Helper functions for constructing resource URLs used during a game

use core_data::game_primitives::{CardType, Rarity, Resonance, School, Side, Sprite};
use core_ui::design;
use core_ui::design::FontColor;
use enumset::EnumSet;
use game_data::card_name::CardMetadata;
use game_data::character_preset::CharacterPreset;
use game_data::special_effects::{
    FantasyEventSounds, FireworksSound, Projectile, SoundEffect, TimedEffect,
};
use protos::riftcaller::{
    AudioClipAddress, CharacterPresetAddress, EffectAddress, FlexColor, ProjectileAddress,
    SpriteAddress,
};

pub mod rexard_images;

/// Possible types of icons which can appear on a card
pub enum CardIconType {
    ProgressCounter,
    Mana,
    Health,
    Attack,
    Shield,
    ProgressRequirement,
    Points,
    Raze,
    PowerCharge,
    StatusQuantity,
}

pub fn side_badge(side: Side) -> SpriteAddress {
    SpriteAddress {
        address: match side {
            Side::Covenant => "Rexard/BadgesMegapack/covenant.png",
            Side::Riftcaller => "Rexard/BadgesMegapack/riftcaller.png",
        }
        .to_string(),
    }
}

/// Returns the background scale multiplier to use for a [CardIconType]
pub fn icon_background_scale(icon_type: CardIconType) -> Option<f32> {
    Some(match icon_type {
        CardIconType::Health => 1.5,
        CardIconType::Attack => 1.75,
        CardIconType::Shield => 1.1,
        CardIconType::ProgressRequirement => 0.9,
        CardIconType::Points => 0.35,
        CardIconType::Raze => 1.5,
        _ => 1.0,
    })
}

/// Address for a given [CardIconType]
pub fn card_icon(icon_type: CardIconType) -> SpriteAddress {
    SpriteAddress {
        address: format!(
            "{}.png",
            match icon_type {
                CardIconType::ProgressCounter => {
                    "LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Icons/Number_Icons_Color_3"
                }
                CardIconType::Mana => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Icons/Icon_Mana_Color_01"
                }
                CardIconType::Health => {
                    "LittleSweetDaemon/TCG_Card_Elemental_Design/Heart_Icons/Heart_Icons_Color_5"
                }
                CardIconType::Attack => {
                    "LittleSweetDaemon/TCG_Card_Elemental_Design/Attack_Icons/Attack_Icons_Color_4"
                }
                CardIconType::Shield => {
                    "LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Icons/Number_Icons_Color_6"
                }
                CardIconType::ProgressRequirement => {
                    "LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Back/Number_Back_Color_3"
                }
                CardIconType::Points => {
                    "LittleSweetDaemon/TCG_Card_Elemental_Design/Card_Color_07/Back_Card_Color_07/Back_Card_Color_07_Logo_Crystal"
                }
                CardIconType::Raze => {
                    "Sprites/Raze"
                }
                CardIconType::PowerCharge => {
                    "LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Icons/Number_Icons_Color_1"
                }
                CardIconType::StatusQuantity => {
                    "LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Icons/Number_Icons_Color_2"
                }
            }
        ),
    }
}

/// Address for the back of a card of a given [School]
pub fn card_back(school: School) -> SpriteAddress {
    SpriteAddress {
        address: format!(
            "{}.png",
            match school {
                School::Law => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/LawCardBack"
                }
                School::Shadow => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/ShadowCardBack"
                }
                School::Primal => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/PrimalCardBack"
                }
                School::Beyond => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/BeyondCardBack"
                }
                School::Pact => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/PactCardBack"
                }
                School::Neutral => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/PrimalCardBack"
                }
            }
        ),
    }
}

/// Address for the frame of a card of a given [School]
pub fn card_frame(school: School, full_height: bool) -> SpriteAddress {
    let string = if full_height {
        match school {
            School::Law => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/CardLawFullHeight",
            School::Shadow => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/CardShadowFullHeight"
            }
            School::Primal => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/CardPrimalFullHeight"
            }
            School::Beyond => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/CardBeyondFullHeight"
            }
            School::Pact => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/CardPactFullHeight",
            School::Neutral => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/CardNeutralFullHeight"
            }
        }
    } else {
        match school {
            School::Law => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/LawCardFrame",
            School::Shadow => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/ShadowCardFrame",
            School::Primal => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/PrimalCardFrame",
            School::Beyond => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/BeyondCardFrame",
            School::Pact => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/PactCardFrame",
            School::Neutral => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/NeutralCardFrame",
        }
    };

    SpriteAddress { address: format!("{string}.png") }
}

/// Title font color to use for a given [Resonance] set.
pub fn title_color(set: EnumSet<Resonance>) -> FlexColor {
    match () {
        _ if set.contains(Resonance::Prismatic) => FontColor::PrismaticCardTitle,
        _ if Resonance::basic_resonance_count(set) > 1 => FontColor::MultiResonanceCardTitle,
        _ if set.contains(Resonance::Mortal) => FontColor::MortalCardTitle,
        _ if set.contains(Resonance::Infernal) => FontColor::InfernalCardTitle,
        _ if set.contains(Resonance::Astral) => FontColor::AstralCardTitle,
        _ => FontColor::NormalCardTitle,
    }
    .into()
}

/// Text reference to a named resonance type.
pub fn resonance_string(name: &'static str) -> String {
    let color = match name {
        "mortal" | "Mortal" => FontColor::MortalCardTitle,
        "infernal" | "Infernal" => FontColor::InfernalCardTitle,
        "astral" | "Astral" => FontColor::AstralCardTitle,
        "prismatic" | "Prismatic" => FontColor::PrismaticCardTitle,
        _ => FontColor::NormalCardTitle,
    };
    format!("<b><color={}>{}</color></b>", design::as_hex(color), name)
}

/// Address for an image to display as a background for a card of the given
/// [Resonance].
pub fn title_background(_: EnumSet<Resonance>) -> SpriteAddress {
    SpriteAddress {
        address: "LittleSweetDaemon/TCG_Card_Design/Custom/Title/BlackWhiteFaceTape.png"
            .to_string(),
    }
}

/// Address for the frame of a card in the arena
pub fn arena_frame(_: Side, card_type: CardType, resonance: EnumSet<Resonance>) -> SpriteAddress {
    SpriteAddress {
        address: format!(
            "{}.png",
            match card_type {
                _ if resonance.contains(Resonance::Prismatic) =>
                    "SpriteWay/Icons/Clean Frames/9047",
                _ if Resonance::basic_resonance_count(resonance) > 1 =>
                    "SpriteWay/Icons/Clean Frames/9008",
                _ if resonance.contains(Resonance::Prismatic) =>
                    "SpriteWay/Icons/Clean Frames/9020",
                _ if resonance.contains(Resonance::Prismatic) =>
                    "SpriteWay/Icons/Clean Frames/9054",
                _ if resonance.contains(Resonance::Prismatic) =>
                    "SpriteWay/Icons/Clean Frames/9048",
                CardType::Evocation => "SpriteWay/Icons/Clean Frames/9013",
                CardType::Scheme => "SpriteWay/Icons/Clean Frames/9032",
                CardType::Project => "SpriteWay/Icons/Clean Frames/9025",
                CardType::GameModifier => "SpriteWay/Icons/Clean Frames/9058",
                CardType::Chapter => "SpriteWay/Icons/Clean Frames/9022",
                CardType::Riftcaller => "SpriteWay/Icons/Clean Frames/9040",
                _ => "SpriteWay/Icons/Clean Frames/9062",
            },
        ),
    }
}

pub fn face_down_arena_frame() -> SpriteAddress {
    SpriteAddress { address: "SpriteWay/Icons/Clean Frames/9052.png".to_string() }
}

/// Address for the rarity jewel to display on a card
pub fn jewel(rarity: Rarity) -> SpriteAddress {
    SpriteAddress {
        address: format!(
            "{}.png",
            match rarity {
                Rarity::Common | Rarity::None | Rarity::Basic => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Elf_Color_01"
                }
                Rarity::Uncommon => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_01"
                }
                Rarity::Rare => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Elf_Color_02"
                }
                Rarity::Identity => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_02"
                }
            }
        ),
    }
}

pub fn projectile(projectile: Projectile) -> ProjectileAddress {
    ProjectileAddress {
        address: match projectile {
            Projectile::Projectiles1(number) => {
                format!("HovlStudio/Projectiles1/Projectile {number}.prefab")
            }
            Projectile::Projectiles2(number) => {
                format!("HovlStudio/Projectiles2/Projectile {number}.prefab")
            }
        },
    }
}

pub fn timed_effect(effect: TimedEffect) -> EffectAddress {
    EffectAddress {
        address: match effect {
            TimedEffect::MagicHits(number) => {
                format!("HovlStudio/MagicHits/Hit {number}.prefab")
            }
            TimedEffect::MagicCircles1(number) => {
                format!("HovlStudio/MagicCircles1/Magic circle {number}.prefab")
            }
            TimedEffect::MagicCircles1Looping(path) => {
                format!("HovlStudio/MagicCircles1/Loop version/{path}.prefab")
            }
            TimedEffect::MagicCircles2(number) => {
                format!("HovlStudio/MagicCircles2/Magic circle {number}.prefab")
            }
            TimedEffect::SwordSlashes(number) => {
                format!("HovlStudio/SwordSlashes/Sword Slash {number}.prefab")
            }
        },
    }
}

pub fn sound_effect(effect: SoundEffect) -> AudioClipAddress {
    AudioClipAddress {
        address: match effect {
            SoundEffect::FantasyEvents(events) => format!(
                "{}.wav",
                match events {
                    FantasyEventSounds::Positive1 => {
                        "Cafofo/Fantasy Music Pack Vol 1/Events/Positive Event 01"
                    }
                }
            ),
            SoundEffect::Fireworks(firework) => format!(
                "{}.wav",
                match firework {
                    FireworksSound::RocketExplodeLarge => {
                        "Universal Sound FX/FIREWORKS/FIREWORKS_Rocket_Explode_Large_RR1_mono"
                    }
                    FireworksSound::RocketExplode => {
                        "Universal Sound FX/FIREWORKS/FIREWORKS_Rocket_Explode_RR1_mono"
                    }
                }
            ),
            SoundEffect::LightMagic(name) => format!("WowSound/Light Magic/{name}.wav"),
            SoundEffect::WaterMagic(name) => format!("WowSound/Water Magic/{name}.wav"),
        },
    }
}

pub fn character_preset(preset: CharacterPreset) -> CharacterPresetAddress {
    CharacterPresetAddress {
        address: match preset {
            CharacterPreset::Riftcaller => "CharacterPresets/Riftcaller.asset",
            CharacterPreset::Covenant => "CharacterPresets/Covenant.asset",
        }
        .to_string(),
    }
}

pub fn ability_title_background() -> SpriteAddress {
    SpriteAddress {
        address: "LittleSweetDaemon/TCG_Card_Design/Custom/Title/TokenTitleBackground.png"
            .to_string(),
    }
}

pub fn riftcaller_card(_: CardMetadata, name: impl Into<String>) -> Sprite {
    Sprite { address: format!("Cards/Riftcaller/{}.png", name.into()) }
}

pub fn covenant_card(_: CardMetadata, name: impl Into<String>) -> Sprite {
    Sprite { address: format!("Cards/Covenant/{}.png", name.into()) }
}

pub fn chapter(_: CardMetadata, name: impl Into<String>) -> Sprite {
    Sprite { address: format!("Cards/Covenant/Chapters/{}.png", name.into()) }
}

pub fn misc_card(name: impl Into<String>, full_art: bool) -> Sprite {
    Sprite {
        address: format!(
            "Cards/{}/{}.png",
            if full_art { "MiscFullArt" } else { "Misc" },
            name.into()
        ),
    }
}
