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

use crate::card_definition::CardDefinition;
use crate::card_name::CardName;
use crate::card_state::{CardData, CardState};
use crate::game::GameState;
use crate::primitives::{ActionCount, BreachValue, CardId, DamageAmount, ManaValue};

pub fn trigger(name: Token, effect: Vec<Text2>) -> Vec<Text2> {
    vec![Text2::NamedTrigger(name, effect)]
}

pub fn encounter_ability_text(cost: Vec<Text2>, effect: Vec<Text2>) -> Vec<Text2> {
    vec![Text2::EncounterAbility { cost, effect }]
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Text2 {
    Children(Vec<Self>),
    NamedTrigger(Token, Vec<Text2>),
    Activated { cost: Vec<Text2>, effect: Vec<Text2> },
    EncounterAbility { cost: Vec<Text2>, effect: Vec<Text2> },
    Literal(String),
    Reminder(String),
    Token(Token),
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Token {
    ManaSymbol,
    Mana(ManaValue),
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
    SuccessfulRaid,
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

impl Token {
    pub fn is_keyword(&self) -> bool {
        matches!(self, Self::Breach(_) | Self::LevelUp | Self::Trap | Self::Construct)
    }
}

impl From<&str> for Text2 {
    fn from(s: &str) -> Self {
        Self::Literal(s.to_owned())
    }
}

impl From<u32> for Text2 {
    fn from(v: u32) -> Self {
        Self::Token(Token::Number(v))
    }
}

impl From<Token> for Text2 {
    fn from(k: Token) -> Self {
        Self::Token(k)
    }
}

impl From<Vec<Text2>> for Text2 {
    fn from(children: Vec<Text2>) -> Self {
        Self::Children(children)
    }
}

/// Provides the context in which rules text is being evaluated, i.e. during an
/// active game or in a deck editor.
pub enum RulesTextContext<'a> {
    Default(&'a CardDefinition),
    Game(&'a GameState, &'a CardState),
}

impl<'a> RulesTextContext<'a> {
    pub fn card_name(&self) -> CardName {
        match self {
            RulesTextContext::Default(definition) => definition.name,
            RulesTextContext::Game(_, card) => card.name,
        }
    }

    pub fn card_data(&self) -> Option<&CardData> {
        match self {
            RulesTextContext::Default(_) => None,
            RulesTextContext::Game(_, card) => Some(&card.data),
        }
    }

    /// Invokes the provided `game` function to product a value in the active
    /// game context, otherwise returns some `default`.
    pub fn query_or<T>(&self, default: T, game: impl Fn(&GameState, CardId) -> T) -> T {
        match self {
            RulesTextContext::Default(_) => default,
            RulesTextContext::Game(state, card) => game(state, card.id),
        }
    }
}
