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

use adapters::response_builder::ResponseBuilder;
use anyhow::Result;
use game_data::game_state::GameState;
use game_data::game_updates::{GameAnimation, TargetedInteraction};
use game_data::primitives::{AbilityId, CardId, GameObjectId, Milliseconds, RoomId, Side};
use game_data::special_effects::{
    FantasyEventSounds, FireworksSound, Projectile, ProjectileData, SoundEffect, SpecialEffect,
    TimedEffect, TimedEffectData,
};
use game_data::utils;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::play_effect_position::EffectPosition;
use protos::spelldawn::{
    CreateTokenCardCommand, DelayCommand, DisplayGameMessageCommand, FireProjectileCommand,
    FlexColor, GameMessageType, GameObjectMove, MoveGameObjectsCommand, MusicState,
    PlayEffectCommand, PlayEffectPosition, PlaySoundCommand, RoomVisitType,
    SetCardMovementEffectCommand, SetMusicCommand, TimeValue, VisitRoomCommand,
};
use rules::CardDefinitionExt;
use {adapters, assets};

use crate::{card_sync, positions};

pub fn render(
    builder: &mut ResponseBuilder,
    update: &GameAnimation,
    snapshot: &GameState,
) -> Result<()> {
    match update {
        GameAnimation::StartTurn(side) => start_turn(builder, *side),
        GameAnimation::PlayCard(side, card_id) => {
            if builder.user_side == side.opponent() {
                show_cards(builder, &vec![*card_id], ShowCards::default())
            }
        }
        GameAnimation::CustomEffects(effects) => play_special_effects(builder, effects),
        GameAnimation::AbilityActivated(side, ability_id) => {
            if *side != builder.user_side {
                show_ability(builder, snapshot, *ability_id);
            }
        }
        GameAnimation::AbilityTriggered(ability_id, effects) => {
            show_ability(builder, snapshot, *ability_id);
            play_special_effects(builder, effects)
        }
        GameAnimation::DrawCards(side, cards) => {
            if builder.user_side == *side {
                show_cards(builder, cards, ShowCards::default())
            }
        }
        GameAnimation::ShuffleIntoDeck => {
            // No animation, just acts as a snapshot point.
        }
        GameAnimation::UnveilCard(card_id) => show_cards(
            builder,
            &vec![*card_id],
            ShowCards { show_if_prominent: true, ..ShowCards::default() },
        ),
        GameAnimation::SummonMinion(card_id) => {
            if builder.user_side == Side::Champion {
                show_cards(
                    builder,
                    &vec![*card_id],
                    ShowCards { show_if_prominent: true, ..ShowCards::default() },
                )
            }
        }
        GameAnimation::LevelUpRoom(room_id, initiated_by) => {
            if initiated_by.is_ability() || builder.user_side == Side::Champion {
                // Animation is not required for the Overlord's own 'level up room' action, it's
                // handled by the client's optimistic animation system.
                level_up_room(builder, *room_id)
            }
        }
        GameAnimation::InitiateRaid(room_id, initiated_by) => {
            if initiated_by.is_ability() || builder.user_side == Side::Overlord {
                // Animation is not required for the Champion's own 'level up room' action, it's
                // handled by the client's optimistic animation system.
                initiate_raid(builder, *room_id)
            }
        }
        GameAnimation::CombatInteraction(interaction) => {
            combat_interaction(builder, snapshot, *interaction)
        }
        GameAnimation::AccessSanctumCards(cards) => access_sanctum_cards(builder, cards),
        GameAnimation::ScoreCard(_, card_id) => score_card(builder, *card_id),
        GameAnimation::GameOver(_) => {}
        GameAnimation::BrowserSubmitted => {}
        GameAnimation::ShowPlayCardBrowser(_) => {}
    }
    Ok(())
}

fn start_turn(builder: &mut ResponseBuilder, side: Side) {
    builder.push(Command::DisplayGameMessage(DisplayGameMessageCommand {
        message_type: match side {
            Side::Overlord => GameMessageType::Dusk.into(),
            Side::Champion => GameMessageType::Dawn.into(),
        },
    }))
}

#[derive(Default)]
struct ShowCards {
    /// If `show_if_prominent` is false, the cards will be skipped if they're
    /// already party of prominent center-screen browser like the raid
    /// display.
    show_if_prominent: bool,
    /// How long to show the cards for. If not specified, cards are shown for
    /// 2000ms if there are >= 4 and 1000ms otherwise.
    milliseconds: Option<u32>,
    /// Whether to always use the large revealed card browser. If false, this
    /// browser is only used if >= 4 cards are shown.
    large_browser: bool,
}

/// An animation to place the indicated cards center-screen.
fn show_cards(builder: &mut ResponseBuilder, cards: &Vec<CardId>, options: ShowCards) {
    let is_large = cards.len() >= 3;
    builder.push(Command::MoveGameObjects(MoveGameObjectsCommand {
        moves: cards
            .iter()
            // Skip animation for cards that are already in a prominent interface position
            .filter(|card_id| !in_display_position(builder, **card_id) || options.show_if_prominent)
            .enumerate()
            .map(|(i, card_id)| GameObjectMove {
                id: Some(adapters::game_object_identifier(builder, *card_id)),
                position: Some(positions::for_sorting_key(
                    i as u32,
                    positions::revealed_cards(options.large_browser || is_large),
                )),
            })
            .collect(),
        disable_animation: !builder.state.animate,
        delay: Some(adapters::milliseconds(options.milliseconds.unwrap_or({
            if is_large {
                2000
            } else {
                1000
            }
        }))),
    }))
}

fn in_display_position(builder: &ResponseBuilder, card_id: CardId) -> bool {
    utils::is_true(|| {
        Some(matches!(
            builder
                .last_snapshot_positions
                .get(&adapters::card_identifier(card_id))?
                .position
                .as_ref()?,
            Position::Staging(_) | Position::Raid(_) | Position::Browser(_) | Position::Revealed(_)
        ))
    })
}

fn show_ability(builder: &mut ResponseBuilder, snapshot: &GameState, ability_id: AbilityId) {
    let mut card = card_sync::ability_card_view(builder, snapshot, ability_id, None);
    card.card_position = Some(positions::for_ability(snapshot, ability_id, positions::staging()));

    builder.push(Command::CreateTokenCard(CreateTokenCardCommand {
        card: Some(card),
        animate: builder.state.animate,
    }));

    builder.push(delay(1500));
}

fn level_up_room(commands: &mut ResponseBuilder, target: RoomId) {
    commands.push(Command::VisitRoom(VisitRoomCommand {
        initiator: commands.to_player_name(Side::Overlord),
        room_id: adapters::room_identifier(target),
        visit_type: RoomVisitType::LevelUpRoom.into(),
    }));
}

fn initiate_raid(commands: &mut ResponseBuilder, target: RoomId) {
    commands.push(Command::VisitRoom(VisitRoomCommand {
        initiator: commands.to_player_name(Side::Champion),
        room_id: adapters::room_identifier(target),
        visit_type: RoomVisitType::InitiateRaid.into(),
    }));
}

fn combat_interaction(
    builder: &mut ResponseBuilder,
    snapshot: &GameState,
    interaction: TargetedInteraction,
) {
    let mut projectile = &ProjectileData::new(Projectile::Projectiles1(3));
    if let GameObjectId::CardId(card_id) = interaction.source {
        if let Some(data) = &snapshot.card(card_id).definition().config.combat_projectile {
            projectile = data;
        }
    }

    builder.push(fire_projectile(builder, interaction, projectile));
}

fn fire_projectile(
    builder: &ResponseBuilder,
    interaction: TargetedInteraction,
    data: &ProjectileData,
) -> Command {
    Command::FireProjectile(FireProjectileCommand {
        source_id: Some(adapters::game_object_identifier(builder, interaction.source)),
        target_id: Some(adapters::game_object_identifier(builder, interaction.target)),
        projectile: Some(assets::projectile(data.projectile)),
        travel_duration: Some(adapters::time_value(data.travel_time)),
        fire_sound: data.fire_sound.map(assets::sound_effect),
        impact_sound: data.impact_sound.map(assets::sound_effect),
        additional_hit: data.additional_hit.map(assets::timed_effect),
        additional_hit_delay: data
            .additional_hit
            .map(|_| adapters::time_value(data.additional_hit_delay)),
        wait_duration: Some(adapters::time_value(data.wait_duration)),
        hide_on_hit: false,
        jump_to_position: None,
    })
}

fn score_card(builder: &mut ResponseBuilder, card_id: CardId) {
    builder.push(set_music(MusicState::Silent));
    builder.push(play_sound(SoundEffect::FantasyEvents(FantasyEventSounds::Positive1)));
    builder.push(play_timed_effect(
        builder,
        card_id,
        &TimedEffectData::new(TimedEffect::MagicHits(4))
            .duration(Milliseconds(700))
            .arena_effect(false)
            .sound(SoundEffect::Fireworks(FireworksSound::RocketExplodeLarge)),
    ));
    builder.push(play_timed_effect(
        builder,
        card_id,
        &TimedEffectData::new(TimedEffect::MagicHits(4))
            .duration(Milliseconds(300))
            .arena_effect(false)
            .sound(SoundEffect::Fireworks(FireworksSound::RocketExplode)),
    ));
    builder.push(delay(1000));
}

fn access_sanctum_cards(builder: &mut ResponseBuilder, cards: &[CardId]) {
    for card_id in cards {
        builder.push(play_timed_effect(
            builder,
            *card_id,
            &TimedEffectData::new(TimedEffect::MagicHits(15))
                .duration(Milliseconds(100))
                .arena_effect(false)
                .sound(SoundEffect::LightMagic("RPG3_LightMagicEpic_Buff02")),
        ));
    }

    builder.push(delay(1000));
}

fn play_special_effects(builder: &mut ResponseBuilder, effects: &[SpecialEffect]) {
    for effect in effects {
        match effect {
            SpecialEffect::TimedEffect { target, effect } => {
                builder.push(play_timed_effect(builder, *target, effect))
            }
            SpecialEffect::Projectile { interaction, projectile } => {
                builder.push(fire_projectile(builder, *interaction, projectile));
            }
            SpecialEffect::CardMovementEffect { card_id, effect } => {
                builder.push(Command::SetCardMovementEffect(SetCardMovementEffectCommand {
                    card_id: Some(adapters::card_identifier(*card_id)),
                    projectile: Some(assets::projectile(*effect)),
                }))
            }
        }
    }
}

fn play_timed_effect(
    builder: &ResponseBuilder,
    position: impl Into<GameObjectId>,
    effect: &TimedEffectData,
) -> Command {
    Command::PlayEffect(PlayEffectCommand {
        effect: Some(assets::timed_effect(effect.effect)),
        position: Some(PlayEffectPosition {
            effect_position: Some(EffectPosition::GameObject(adapters::game_object_identifier(
                builder,
                position.into(),
            ))),
        }),
        scale: effect.scale,
        duration: Some(adapters::time_value(effect.duration)),
        sound: effect.sound.map(assets::sound_effect),
        arena_effect: effect.arena_effect,
        start_color: effect.effect_color.as_ref().map(|color| FlexColor {
            red: color.red,
            green: color.green,
            blue: color.blue,
            alpha: color.alpha,
        }),
    })
}

pub fn delay(milliseconds: u32) -> Command {
    Command::Delay(DelayCommand { duration: Some(TimeValue { milliseconds }) })
}

fn set_music(music_state: MusicState) -> Command {
    Command::SetMusic(SetMusicCommand { music_state: music_state.into() })
}

fn play_sound(sound: SoundEffect) -> Command {
    Command::PlaySound(PlaySoundCommand { sound: Some(assets::sound_effect(sound)) })
}
