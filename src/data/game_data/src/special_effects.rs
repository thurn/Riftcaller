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

//! Types which describe custom visual & sound effects used during play

use crate::game_updates::TargetedInteraction;
use crate::primitives::{CardId, GameObjectId, Milliseconds};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Projectile {
    /// Hovl Studios projectile number
    Projectiles1(u32),
}

/// A projectile asset and associated behavior data
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProjectileData {
    /// Primary projectile to be fired
    pub projectile: Projectile,
    /// Additional hit effect after primary projectile impact
    pub additional_hit: Option<TimedEffect>,
    /// Sound effect when fired
    pub fire_sound: Option<SoundEffect>,
    /// Sound effect when impacting
    pub impact_sound: Option<SoundEffect>,
    /// Time to reach destination. Defaults to 300ms.
    pub travel_time: Milliseconds,
    /// Delay before showing the additional hit. If provided, the original
    /// projectile Hit effect will be hidden before showing the new hit effect.
    /// Defaults to 100ms.
    pub additional_hit_delay: Milliseconds,
    /// During to wait for the project's impact effect before continuing.
    /// Defaults to 300ms.
    pub wait_duration: Milliseconds,
}

impl ProjectileData {
    pub fn new(projectile: Projectile) -> Self {
        Self {
            projectile,
            additional_hit: None,
            fire_sound: None,
            impact_sound: None,
            travel_time: Milliseconds(300),
            additional_hit_delay: Milliseconds(100),
            wait_duration: Milliseconds(300),
        }
    }

    pub fn additional_hit(mut self, additional_hit: TimedEffect) -> Self {
        self.additional_hit = Some(additional_hit);
        self
    }

    pub fn fire_sound(mut self, fire_sound: SoundEffect) -> Self {
        self.fire_sound = Some(fire_sound);
        self
    }

    pub fn impact_sound(mut self, impact_sound: SoundEffect) -> Self {
        self.impact_sound = Some(impact_sound);
        self
    }

    pub fn travel_time(mut self, travel_time: Milliseconds) -> Self {
        self.travel_time = travel_time;
        self
    }

    pub fn additional_hit_delay(mut self, additional_hit_delay: Milliseconds) -> Self {
        self.additional_hit_delay = additional_hit_delay;
        self
    }

    pub fn wait_duration(mut self, wait_duration: Milliseconds) -> Self {
        self.wait_duration = wait_duration;
        self
    }
}

/// Effect which plays for a short duration and then vanishes
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TimedEffect {
    /// Magic hit number
    MagicHits(u32),
    /// Sword Slash VFX number
    SwordSlashes(u32),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FireworksSound {
    RocketExplodeLarge,
    RocketExplode,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FantasyEventSounds {
    Positive1,
}

/// Plays a sound
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SoundEffect {
    FantasyEvents(FantasyEventSounds),
    Fireworks(FireworksSound),
}

#[derive(Debug, Clone)]
pub struct TimedEffectData {
    pub effect: TimedEffect,
    /// How long to play the effect for. Defaults to 300ms.
    pub duration: Milliseconds,
    /// Sound to play with this effect.
    pub sound: Option<SoundEffect>,
    /// Scaling to apply to the effect.
    pub scale: Option<f32>,
}

impl TimedEffectData {
    pub fn new(effect: TimedEffect) -> Self {
        Self { effect, duration: Milliseconds(300), sound: None, scale: None }
    }

    pub fn duration(mut self, duration: Milliseconds) -> Self {
        self.duration = duration;
        self
    }

    pub fn sound(mut self, sound: SoundEffect) -> Self {
        self.sound = Some(sound);
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = Some(scale);
        self
    }
}

/// Represents a single visual effect or sound effect to play
#[derive(Clone, Debug)]
pub enum SpecialEffect {
    TimedEffect { target: GameObjectId, effect: TimedEffectData },
    Projectile { interaction: TargetedInteraction, projectile: ProjectileData },
    CardMovementEffect { card_id: CardId, effect: Projectile },
}
