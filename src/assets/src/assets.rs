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

//! Helper functions for constructing resource URLs used during a game

pub mod rexard_images;

use core_ui::design::FontColor;
use game_data::character_preset::CharacterPreset;
use game_data::primitives::{CardType, Rarity, Resonance, School, Side};
use game_data::special_effects::{
    FantasyEventSounds, FireworksSound, Projectile, SoundEffect, TimedEffect,
};
use protos::spelldawn::{
    AudioClipAddress, CharacterPresetAddress, EffectAddress, FlexColor, ProjectileAddress,
    SpriteAddress,
};

/// Possible types of icons which can appear on a card
pub enum CardIconType {
    LevelCounter,
    Mana,
    Health,
    Attack,
    Shield,
    LevelRequirement,
    Points,
    Raze,
}

pub fn side_badge(side: Side) -> SpriteAddress {
    SpriteAddress {
        address: match side {
            Side::Overlord => "Rexard/BadgesMegapack/overlord.png",
            Side::Champion => "Rexard/BadgesMegapack/champion.png",
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
        CardIconType::LevelRequirement => 0.9,
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
                CardIconType::LevelCounter => {
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
                CardIconType::LevelRequirement => {
                    "LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Back/Number_Back_Color_3"
                }
                CardIconType::Points => {
                    "LittleSweetDaemon/TCG_Card_Elemental_Design/Card_Color_07/Back_Card_Color_07/Back_Card_Color_07_Logo_Crystal"
                }
                CardIconType::Raze => {
                    "Sprites/Raze"
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
pub fn card_frame(school: School, _: CardType) -> SpriteAddress {
    let string = match school {
        School::Law => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/LawCardFrame",
        School::Shadow => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/ShadowCardFrame",
        School::Primal => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/PrimalCardFrame",
        School::Beyond => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/BeyondCardFrame",
        School::Pact => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/PactCardFrame",
        School::Neutral => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/NeutralCardFrame",
    };

    SpriteAddress { address: format!("{string}.png") }
}

/// Title font color to use for a given [Resonance].
pub fn title_color(resonance: Option<Resonance>) -> FlexColor {
    match resonance {
        None => FontColor::NormalCardTitle,
        Some(Resonance::Mortal) => FontColor::MortalCardTitle,
        Some(Resonance::Infernal) => FontColor::InfernalCardTitle,
        Some(Resonance::Abyssal) => FontColor::AbyssalCardTitle,
        Some(Resonance::Prismatic) => FontColor::PrismaticCardTitle,
        Some(Resonance::Construct) => FontColor::ConstructCardTitle,
    }
    .into()
}

/// Address for an image to display as a background for a card of the given
/// [Resonance].
pub fn title_background(_: Option<Resonance>) -> SpriteAddress {
    SpriteAddress {
        address: "LittleSweetDaemon/TCG_Card_Design/Custom/Title/BlackWhiteFaceTape.png"
            .to_string(),
    }
}

/// Address for the frame of a card in the arena
pub fn arena_frame(side: Side, card_type: CardType, resonance: Option<Resonance>) -> SpriteAddress {
    SpriteAddress {
        address: format!(
            "{}.png",
            match resonance {
                Some(Resonance::Mortal) => "SpriteWay/Icons/Clean Frames/9048",
                Some(Resonance::Infernal) => "SpriteWay/Icons/Clean Frames/9054",
                Some(Resonance::Abyssal) => "SpriteWay/Icons/Clean Frames/9020",
                Some(Resonance::Prismatic) => "SpriteWay/Icons/Clean Frames/9047",
                Some(Resonance::Construct) => "SpriteWay/Icons/Clean Frames/9003",
                None => match card_type {
                    CardType::Evocation => "SpriteWay/Icons/Clean Frames/9013",
                    CardType::Scheme => "SpriteWay/Icons/Clean Frames/9032",
                    CardType::Project => "SpriteWay/Icons/Clean Frames/9025",
                    CardType::Riftcaller =>
                        if side == Side::Overlord {
                            "SpriteWay/Icons/Clean Frames/9022"
                        } else {
                            "SpriteWay/Icons/Clean Frames/9040"
                        },
                    _ => "SpriteWay/Icons/Clean Frames/9062",
                },
            }
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
                Rarity::Common | Rarity::None => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Elf_Color_01"
                }
                Rarity::Rare => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_01"
                }
                Rarity::Exalted => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Elf_Color_02"
                }
                Rarity::Epic => {
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_02"
                }
            }
        ),
    }
}

pub fn projectile(projectile: Projectile) -> ProjectileAddress {
    ProjectileAddress {
        address: match projectile {
            Projectile::Hovl(number) => format!(
                "Hovl Studio/AAA Projectiles Vol 1/Prefabs/Projectiles/Projectile {number}.prefab"
            ),
        },
    }
}

pub fn timed_effect(effect: TimedEffect) -> EffectAddress {
    EffectAddress {
        address: match effect {
            TimedEffect::HovlMagicHit(number) => {
                format!("Hovl Studio/Magic hits/Prefabs/Hit {number}.prefab")
            }
            TimedEffect::HovlSwordSlash(number) => {
                format!("Hovl Studio/Sword slash VFX/Prefabs/Sword Slash {number}.prefab")
            }
        },
    }
}

pub fn sound_effect(effect: SoundEffect) -> AudioClipAddress {
    AudioClipAddress {
        address: format!(
            "{}.wav",
            match effect {
                SoundEffect::FantasyEvents(events) => match events {
                    FantasyEventSounds::Positive1 => {
                        "Cafofo/Fantasy Music Pack Vol 1/Events/Positive Event 01"
                    }
                },
                SoundEffect::Fireworks(firework) => match firework {
                    FireworksSound::RocketExplodeLarge => {
                        "Universal Sound FX/FIREWORKS/FIREWORKS_Rocket_Explode_Large_RR1_mono"
                    }
                    FireworksSound::RocketExplode => {
                        "Universal Sound FX/FIREWORKS/FIREWORKS_Rocket_Explode_RR1_mono"
                    }
                },
            }
        ),
    }
}

pub fn character_preset(preset: CharacterPreset) -> CharacterPresetAddress {
    CharacterPresetAddress {
        address: match preset {
            CharacterPreset::Champion => "CharacterPresets/Champion.asset",
            CharacterPreset::Overlord => "CharacterPresets/Ovelord.asset",
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
