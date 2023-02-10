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

use enum_kinds::EnumKind;

use crate::primitives::{ActionCount, BreachValue, DamageAmount, ManaValue};

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum TextElement {
    Children(Vec<Self>),
    NamedTrigger(TextToken, Vec<TextElement>),
    Activated { cost: Vec<TextElement>, effect: Vec<TextElement> },
    EncounterAbility { cost: Vec<TextElement>, effect: Vec<TextElement> },
    Literal(String),
    Reminder(String),
    Token(TextToken),
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumKind)]
#[enum_kind(TextTokenKind, derive(Ord, PartialOrd))]
pub enum TextToken {
    ManaSymbol,
    Mana(ManaValue),
    ManaMinus(ManaValue),
    ActionSymbol,
    Actions(ActionCount),
    Number(u32),
    Plus(u32),
    EncounterBoostCost,
    EncounterBoostBonus,
    Attack,
    Health,
    Gain,
    Lose,
    Play,
    Dawn,
    Dusk,
    Score,
    Combat,
    Encounter,
    Unveil,
    BeginARaid,
    StoreMana(ManaValue),
    TakeMana(ManaValue),
    DealDamage(DamageAmount),
    TakeDamage(DamageAmount),
    InnerRoom,
    OuterRoom,
    Sanctum,
    Vault,
    Crypts,
    Breach(BreachValue),
    LevelUp,
    Trap,
    Construct,
}

impl TextToken {
    pub fn is_keyword(&self) -> bool {
        matches!(self, Self::Breach(_) | Self::LevelUp | Self::Trap | Self::Construct)
    }

    pub fn kind(&self) -> TextTokenKind {
        self.into()
    }
}

impl From<&str> for TextElement {
    fn from(s: &str) -> Self {
        Self::Literal(s.to_owned())
    }
}

impl From<u32> for TextElement {
    fn from(v: u32) -> Self {
        Self::Token(TextToken::Number(v))
    }
}

impl From<TextToken> for TextElement {
    fn from(k: TextToken) -> Self {
        Self::Token(k)
    }
}

impl From<Vec<TextElement>> for TextElement {
    fn from(children: Vec<TextElement>) -> Self {
        Self::Children(children)
    }
}