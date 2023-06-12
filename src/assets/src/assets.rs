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
use game_data::primitives::{CardType, Lineage, Rarity, School, Side, Sprite};
use game_data::special_effects::{
    FantasyEventSounds, FireworksSound, Projectile, SoundEffect, TimedEffect,
};
use protos::spelldawn::{
    AudioClipAddress, EffectAddress, FlexColor, ProjectileAddress, SpriteAddress,
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
}

/// Returns the background scale multiplier to use for a [CardIconType]
pub fn background_scale(icon_type: CardIconType) -> Option<f32> {
    Some(match icon_type {
        CardIconType::Health => 1.5,
        CardIconType::Attack => 1.75,
        CardIconType::Shield => 1.1,
        CardIconType::LevelRequirement => 0.9,
        CardIconType::Points => 0.35,
        _ => 1.0,
    })
}

pub fn side_badge(side: Side) -> Sprite {
    Sprite {
        address: match side {
            Side::Overlord => "Rexard/BadgesMegapack/overlord.png",
            Side::Champion => "Rexard/BadgesMegapack/champion.png",
        }
        .to_string(),
    }
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
            }
        ),
    }
}

/// Address for the frame of a player's leader card image
pub fn leader_card_frame(side: Side) -> SpriteAddress {
    SpriteAddress { address: format!("{}.png", leader_card_frame_string(side)) }
}

fn leader_card_frame_string(side: Side) -> &'static str {
    match side {
        Side::Overlord => "SpriteWay/Icons/Fantasy Player Frames/50002",
        Side::Champion => "SpriteWay/Icons/Fantasy Player Frames/50003",
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
                    "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/BloodCardBack"
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
        School::Pact => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/BloodCardFrame",
        School::Neutral => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Custom/NeutralCardFrame",
    };

    SpriteAddress { address: format!("{string}.png") }
}

pub fn ability_card_frame(side: Side) -> SpriteAddress {
    SpriteAddress {
        address: format!(
            "{}.png",
            match side {
                Side::Overlord => "LittleSweetDaemon/TCG_Card_Design/Custom/OverlordFront",
                Side::Champion => "LittleSweetDaemon/TCG_Card_Design/Custom/ChampionFront",
            }
        ),
    }
}

/// Title font color to use for a given [Lineage].
pub fn title_color(lineage: Option<Lineage>) -> FlexColor {
    match lineage {
        None => FontColor::NormalCardTitle,
        Some(Lineage::Mortal) => FontColor::MortalCardTitle,
        Some(Lineage::Infernal) => FontColor::InfernalCardTitle,
        Some(Lineage::Abyssal) => FontColor::AbyssalCardTitle,
        Some(Lineage::Prismatic) => FontColor::PrismaticCardTitle,
        Some(Lineage::Construct) => FontColor::ConstructCardTitle,
    }
    .into()
}

/// Address for an image to display as a background for a card of the given
/// [Lineage].
pub fn title_background(_: Option<Lineage>) -> SpriteAddress {
    SpriteAddress {
        address: "LittleSweetDaemon/TCG_Card_Design/Custom/Title/BlackWhiteFaceTape.png"
            .to_string(),
    }
}

/// Address for the frame of a card in the arena
pub fn arena_frame(_: Side, card_type: CardType, lineage: Option<Lineage>) -> SpriteAddress {
    SpriteAddress {
        address: format!(
            "{}.png",
            match lineage {
                Some(Lineage::Mortal) => "SpriteWay/Icons/Clean Frames/9048",
                Some(Lineage::Infernal) => "SpriteWay/Icons/Clean Frames/9054",
                Some(Lineage::Abyssal) => "SpriteWay/Icons/Clean Frames/9020",
                Some(Lineage::Prismatic) => "SpriteWay/Icons/Clean Frames/9047",
                Some(Lineage::Construct) => "SpriteWay/Icons/Clean Frames/9003",
                None => match card_type {
                    CardType::Artifact => "SpriteWay/Icons/Clean Frames/9013",
                    CardType::Scheme => "SpriteWay/Icons/Clean Frames/9032",
                    CardType::Project => "SpriteWay/Icons/Clean Frames/9025",
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

pub fn fantasy_class_image(name: &'static str, sex: &'static str) -> Sprite {
    Sprite {
        address: format!("ForrestIml/FantasyClasses1/Characters/{name}/{name}_{sex}_Stock Art.png"),
    }
}

pub fn fantasy_class_portrait(side: Side, name: impl Into<String>) -> Sprite {
    Sprite {
        address: format!(
            "ForrestIml/FantasyClasses1/UI_Portraits/{}_Square/UI_Portrait_{}_Square_{}.png",
            match side {
                Side::Overlord => "Silver",
                Side::Champion => "Gold",
            },
            match side {
                Side::Overlord => "Silver",
                Side::Champion => "Gold",
            },
            name.into()
        ),
    }
}

pub enum EnvironmentType {
    CastlesTowersKeeps,
    DungeonsShrinesAltars,
}

pub fn environments(environment_type: EnvironmentType, path: &'static str) -> Sprite {
    Sprite {
        address: format!(
            "TPR/EnvironmentsHQ/{}/Images/{}.png",
            match environment_type {
                EnvironmentType::CastlesTowersKeeps => "Castles, Towers & Keeps",
                EnvironmentType::DungeonsShrinesAltars => "Dungeons, Shrines & Altars",
            },
            path
        ),
    }
}
