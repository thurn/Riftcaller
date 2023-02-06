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

use crate::card_definition::Cost;
use crate::primitives::{ActionCount, BreachValue, DamageAmount, ManaValue};

pub fn trigger(name: Token, effect: Vec<Text2>) -> Text2 {
    Text2::KeywordTrigger(name, effect)
}

pub fn activation(effect: Vec<Text2>) -> Text2 {
    Text2::Literal("Hello".to_string())
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Text2 {
    Children(Vec<Self>),
    KeywordTrigger(Token, Vec<Self>),
    ActivationCost(Vec<Self>),
    BoostCost(Vec<Self>),
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
    Then,
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

impl<T> From<Cost<T>> for Text2 {
    fn from(cost: Cost<T>) -> Self {
        let mut result = vec![];
        if let Some(mana) = cost.mana {
            result.push(Self::Token(Token::Mana(mana)))
        }

        if cost.actions > 1 {
            result.push(Self::Token(Token::Actions(cost.actions)));
        }

        Self::ActivationCost(result)
    }
}
