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

pub mod tutorial_actions;

use core_ui::design::{self, FontColor};
use core_ui::icons;
use data::card_name::CardName;
use data::game_actions::CardTarget;
use data::primitives::{Milliseconds, RoomId, Side};
use data::tutorial_data::{
    SpeechBubble, Toast, Tooltip, TooltipAnchor, TutorialDisplay, TutorialMessageKey,
    TutorialMessageTrigger, TutorialOpponentAction, TutorialSequence, TutorialStep,
    TutorialTrigger,
};
use once_cell::sync::Lazy;

pub const PLAYER_SIDE: Side = Side::Champion;
pub const OPPONENT_SIDE: Side = Side::Overlord;

/// Definition for the game tutorial
pub static SEQUENCE: Lazy<TutorialSequence> = Lazy::new(|| {
    TutorialSequence {

        /// The first few turns of the tutorial game are pre-scripted and
        /// defined here
        steps: vec![
            TutorialStep::SetHand(Side::Overlord, vec![CardName::Frog]),
            TutorialStep::SetHand(Side::Champion, vec![CardName::EldritchSurge]),
            TutorialStep::SetTopOfDeck(Side::Overlord, vec![CardName::Captain, CardName::Machinate]),
            TutorialStep::SetTopOfDeck(Side::Champion, vec![CardName::SimpleAxe]),
            TutorialStep::KeepOpeningHand(Side::Champion),
            TutorialStep::KeepOpeningHand(Side::Overlord),
            TutorialStep::OpponentAction(TutorialOpponentAction::DrawCard),
            TutorialStep::OpponentAction(TutorialOpponentAction::PlayCard(
                CardName::Machinate,
                CardTarget::Room(RoomId::RoomA),
            )),
            TutorialStep::OpponentAction(TutorialOpponentAction::PlayCard(
                CardName::Captain,
                CardTarget::Room(RoomId::RoomA),
            )),
            TutorialStep::Display(vec![
                opponent_say("Surrender to the night!", Milliseconds(0)),
                user_say("Your tyranny ends here, Vaughn!", Milliseconds(4000)),
                toast_at(
                    format!(
                        "Tips: <b>Mana</b> ({}) lets you play cards and use weapons. It persists between turns.",
                        icons::MANA
                    ),
                    Milliseconds(8000),
                ),
                user_say("I should play a card...", Milliseconds(30_000)),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::PlayAnyCard]),
            TutorialStep::Display(vec![
                user_say("No evil shall stand against my valor.", Milliseconds(4000)),
                toast_at(
                    format!(
                        "Playing cards from your hand costs one {} (<b>action point</b>).",
                        icons::ACTION
                    ),
                    Milliseconds(0),
                ),
                user_say("I should play a card...", Milliseconds(20_000)),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::PlayAnyCard]),
            // User -> 4 mana
            TutorialStep::Display(vec![
                user_say("My weapon is ready.", Milliseconds(0)),
                user_say("I should investigate that room...", Milliseconds(4000)),
                toast_at(
                    format!("You can spend {} to start a <b>raid</b> and explore a <b>room</b> of the enemy's dungeon.", icons::ACTION),
                    Milliseconds(6000),
                ),
                tooltip(
                    "Drag portrait here",
                    TooltipAnchor::RaidRoom(RoomId::RoomA),
                    Milliseconds(8000),
                ),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::InitiateRaid(RoomId::RoomA)]),
            TutorialStep::Display(vec![
                toast_at(
                    "To get past a defending minion, you must deal damage to it equal to its <b>health</b>.",
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::UseWeapon {
                weapon: CardName::SimpleAxe,
                target: CardName::Captain,
            }]),
            TutorialStep::Display(vec![
                toast_at(
                    "Once you access a room, you can <b>score</b> a card inside.",
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::ScoreAccessedCard(
                CardName::Machinate,
            )]),
            TutorialStep::Display(vec![
                toast_at(
                    "Scoring <b>scheme</b> cards in rooms gives you points. The first player to reach 100 points wins!",
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::Display(vec![opponent_say("Curse you!", Milliseconds(0))]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::EndRaid]),
            // User -> 4 mana
            TutorialStep::SetTopOfDeck(Side::Overlord, vec![CardName::GatheringDark]),
            TutorialStep::OpponentAction(TutorialOpponentAction::GainMana),
            TutorialStep::OpponentAction(TutorialOpponentAction::GainMana),
            TutorialStep::OpponentAction(TutorialOpponentAction::GainMana),
            // Opponent -> 5 mana
            TutorialStep::SetTopOfDeck(
                Side::Champion,
                vec![CardName::ArcaneRecovery, CardName::Lodestone],
            ),
            TutorialStep::Display(vec![
                user_say("I need more mana...", Milliseconds(0)),
                toast_at(
                    format!("You can spend {} to gain 1{}.", icons::ACTION, icons::MANA),
                    Milliseconds(4000),
                ),
                tooltip("Tap to gain mana", TooltipAnchor::GainMana, Milliseconds(4_000)),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::GainMana]),
            // User -> 5 mana
            TutorialStep::Display(vec![
                user_say("You'll pay for what you did.", Milliseconds(0)),
                toast_at("Now you can play this card", Milliseconds(4000)),
                user_say("I should play a card...", Milliseconds(20_000))
                ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::PlayCard(
                CardName::ArcaneRecovery,
                CardTarget::None,
            )]),
            // User -> 9 mana
            TutorialStep::Display(vec![
                user_say("I should draw another card...", Milliseconds(0)),
                toast_at(
                    format!("You can spend {} to draw a card.", icons::ACTION),
                    Milliseconds(4000),
                ),
                tooltip("Tap to draw card", TooltipAnchor::DrawCard, Milliseconds(4000)),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::DrawCard]),
            TutorialStep::SetTopOfDeck(Side::Overlord, vec![CardName::Devise]),
            TutorialStep::OpponentAction(TutorialOpponentAction::PlayCard(
                CardName::GatheringDark,
                CardTarget::None,
            )),
            TutorialStep::OpponentAction(TutorialOpponentAction::PlayCard(
                CardName::Devise,
                CardTarget::Room(RoomId::RoomA),
            )),
            TutorialStep::OpponentAction(TutorialOpponentAction::PlayCard(
                CardName::Frog,
                CardTarget::Room(RoomId::RoomA),
            )),
            TutorialStep::SetTopOfDeck(
                Side::Champion,
                vec![
                    // We set up the deck in such a way that an Abyssal weapon
                    // cannot be drawn to defeat the Frog, in order to
                    // illustrate a failed raid.

                    CardName::SimpleHammer,
                    CardName::Contemplate,
                    CardName::ArcaneRecovery,
                    CardName::EldritchSurge,
                    CardName::SimpleBlade,
                    CardName::SimpleSpear,
                    CardName::SimpleAxe,
                    CardName::SimpleHammer,
                    CardName::Contemplate,
                    CardName::ArcaneRecovery,
                    CardName::EldritchSurge,
                    CardName::SimpleBlade,
                    CardName::SimpleSpear,
                    CardName::SimpleAxe,
                    CardName::SimpleHammer,
                    CardName::Contemplate,
                    CardName::SimpleBlade,
                    CardName::SimpleSpear,
                    CardName::SimpleAxe,
                ],
            ),
            TutorialStep::Display(vec![
                user_say("Your minions can't keep me out of that room.", Milliseconds(0)),
            ]),
            TutorialStep::OpponentAction(TutorialOpponentAction::GainMana),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::InitiateRaid(RoomId::RoomA)]),
            TutorialStep::Display(vec![
                toast_at(
                    format!("A <color={}>Mortal</color> weapon cannot damage an <color={}>Abyssal</color> minion. A matching weapon is required!",
                    design::as_hex(FontColor::MortalCardTitle),
                    design::as_hex(FontColor::AbyssalCardTitle)),
                    Milliseconds(0),
                ),
            ]),
        ],

        // From this point on, we transition to running a normal game with
        // contextual help messages, instead of pre-scripting everything.
        messages: vec![
            TutorialMessageTrigger {
                key: TutorialMessageKey::PlayInfernalWeapon,
                trigger: TutorialTrigger::PlayCard(CardName::SimpleHammer, CardTarget::None),
                display: vec![
                    toast(
                        format!("There are three different kinds of weapons: <color={}>Mortal</color>, <color={}>Infernal</color>, and <color={}>Abyssal</color>.",
                        design::as_hex(FontColor::MortalCardTitle),
                        design::as_hex(FontColor::InfernalCardTitle),
                        design::as_hex(FontColor::AbyssalCardTitle)),
                    )
                ]
            },
            TutorialMessageTrigger {
                key: TutorialMessageKey::PlayAbilityCard,
                trigger: TutorialTrigger::PlayCard(CardName::Lodestone, CardTarget::None),
                display: vec![toast("Some cards in play have <b>activated abilities</b> which show up in your hand")]
            }
        ]
    }
});

fn user_say(text: impl Into<String>, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::SpeechBubble(SpeechBubble { text: text.into(), side: Side::Champion, delay })
}

fn opponent_say(text: impl Into<String>, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::SpeechBubble(SpeechBubble { text: text.into(), side: Side::Overlord, delay })
}

fn tooltip(text: impl Into<String>, anchor: TooltipAnchor, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::Tooltip(Tooltip { text: text.into(), anchor, delay })
}

fn toast(text: impl Into<String>) -> TutorialDisplay {
    TutorialDisplay::Toast(Toast {
        text: text.into(),
        delay: Milliseconds(0),
        hide_after: Some(Milliseconds(10_000)),
    })
}

fn toast_at(text: impl Into<String>, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::Toast(Toast { text: text.into(), delay, hide_after: None })
}
