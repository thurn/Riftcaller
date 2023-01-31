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
use game_data::card_name::CardName;
use game_data::game_actions::CardTarget;
use game_data::primitives::{Milliseconds, RoomId, Side};
use game_data::tutorial_data::{
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
            TutorialStep::AddGameModifiers(vec![
                CardName::TutorialDisableDrawAction,
                CardName::TutorialDisableGainMana,
                CardName::TutorialDisableRaidSanctum,
                CardName::TutorialDisableRaidVault,
                CardName::TutorialDisableRaidCrypts,
                CardName::TutorialDisableRaidOuter,
                CardName::TutorialDisableRaidContinue,
                CardName::TutorialDisableEndRaid
            ]),
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
                toast_at(
                    format!(
                        "Tips: <b>Mana</b> ({}) lets you play cards and use weapons. It persists between turns.",
                        icons::MANA
                    ),
                    Milliseconds(0),
                ),
                opponent_say("Surrender to the night!", Milliseconds(4000)),
                user_say("Your tyranny ends here, Vaughn!", Milliseconds(8000)),
                user_say("I should play a card...", Milliseconds(30_000)),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::PlayAnyCard]),
            TutorialStep::Display(vec![
                user_say("No evil shall stand against my valor.", Milliseconds(4000)),
                toast_at(
                    format!(
                        "Playing cards from your hand costs one {} (action point).",
                        icons::ACTION
                    ),
                    Milliseconds(0),
                ),
                user_say("I should play a card...", Milliseconds(20_000)),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::PlayAnyCard]),
            // User -> 4 mana
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableRaidOuter,
            ]),
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
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableRaidContinue,
            ]),
            TutorialStep::Display(vec![
                toast_at(
                    "Once you access a room, you can <b>score</b> a card inside.",
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::ScoreAccessedCard(
                CardName::Machinate,
            )]),
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableEndRaid,
            ]),
            TutorialStep::Display(vec![
                toast_at(
                    "Scoring <b>scheme</b> cards in rooms gives you points. The first player to reach 100 points wins!",
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::Display(vec![opponent_say("Curse you!", Milliseconds(0))]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::SuccessfullyEndRaid]),
            // User -> 4 mana
            TutorialStep::SetTopOfDeck(Side::Overlord, vec![CardName::GatheringDark]),
            TutorialStep::OpponentAction(TutorialOpponentAction::GainMana),
            TutorialStep::OpponentAction(TutorialOpponentAction::GainMana),
            TutorialStep::OpponentAction(TutorialOpponentAction::GainMana),
            // Opponent -> 5 mana
            TutorialStep::SetTopOfDeck(
                Side::Champion,
                vec![CardName::Lodestone, CardName::ArcaneRecovery],
            ),
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableGainMana,
            ]),
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
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableDrawAction,
            ]),
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

                    CardName::Contemplate,
                    CardName::EldritchSurge,
                    CardName::ArcaneRecovery,
                    CardName::SimpleBlade,
                    CardName::SimpleSpear,
                    CardName::SimpleAxe,
                    CardName::SimpleBlade,
                    CardName::SimpleHammer,
                    CardName::Lodestone,
                    CardName::Contemplate,
                    CardName::EldritchSurge,
                    CardName::SimpleBlade,
                    CardName::SimpleSpear,
                    CardName::SimpleSpear,
                    CardName::ArcaneRecovery,
                    CardName::SimpleAxe,
                    CardName::SimpleHammer,
                    CardName::Lodestone,
                    CardName::Contemplate,
                    CardName::SimpleHammer,
                ],
            ),
            TutorialStep::DefaultOpponentAction(TutorialOpponentAction::GainMana),
            TutorialStep::Display(vec![
                user_say("You can't keep me out of that room.", Milliseconds(0)),
                user_say_recurring("I should return to that room.", Milliseconds(10_000)),
                tooltip_recurring(
                    "Drag portrait here",
                    TooltipAnchor::RaidRoom(RoomId::RoomA),
                    Milliseconds(15_000),
                ),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::InitiateRaid(RoomId::RoomA)]),
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableRaidVault,
                CardName::TutorialDisableRaidSanctum,
                CardName::TutorialDisableRaidCrypts
            ]),
            TutorialStep::Display(vec![
                toast_at(
                    format!("A <color={}>Mortal</color> weapon cannot damage an <color={}>Abyssal</color> minion. A matching weapon is required!",
                    design::as_hex(FontColor::MortalCardTitle),
                    design::as_hex(FontColor::AbyssalCardTitle)),
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::UseNoWeapon]),
            TutorialStep::Display(vec![
                user_say("There's got to be another way!", Milliseconds(0)),
                user_say_recurring("I should search the Vault.", Milliseconds(8_000)),
                toast_at(
                    "You can raid the <b>Vault</b> and attempt to score cards on top of your opponent's deck.",
                    Milliseconds(2000)),
                tooltip_recurring(
                    "Drag portrait here",
                    TooltipAnchor::RaidRoom(RoomId::Vault),
                    Milliseconds(10_000),
                ),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::InitiateRaid(RoomId::Vault)]),
            TutorialStep::SetTopOfDeck(Side::Champion, vec![CardName::SimpleClub]),
            TutorialStep::Display(vec![
                toast_at(
                    "If the top card of your opponent's deck is a <b>scheme</b> card, you can score it for points!",
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::SetTopOfDeck(Side::Overlord, vec![CardName::Machinate, CardName::Conspire]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::SuccessfullyEndRaid]),
            TutorialStep::Display(vec![
                toast_at(
                    format!("Now you just need to find an <color={}>Abyssal</color> weapon",
                    design::as_hex(FontColor::AbyssalCardTitle)),
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::OpponentAction(TutorialOpponentAction::LevelUpRoom(
                RoomId::RoomA,
            )),
            TutorialStep::OpponentAction(TutorialOpponentAction::LevelUpRoom(
                RoomId::RoomA,
            )),
            TutorialStep::OpponentAction(TutorialOpponentAction::LevelUpRoom(
                RoomId::RoomA,
            )),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::InitiateRaid(RoomId::RoomA)]),
            TutorialStep::Display(vec![
                toast_at(
                    format!("An <color={}>Abyssal</color> weapon can only damage an <color={}>Abyssal</color> minion.",
                    design::as_hex(FontColor::AbyssalCardTitle),
                    design::as_hex(FontColor::AbyssalCardTitle)),
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::AwaitPlayerActions(vec![TutorialTrigger::SuccessfullyEndRaid]),
            TutorialStep::Display(vec![
                user_say("Time to end this.", Milliseconds(0)),
                user_say_recurring("I need to attack the Sanctum", Milliseconds(8_000)),
                toast_at(
                    "You can raid the <b>Sanctum</b> to access a random card from your opponent's hand",
                    Milliseconds(2000)),
                tooltip_recurring(
                    "Drag portrait here",
                    TooltipAnchor::RaidRoom(RoomId::Sanctum),
                    Milliseconds(10_000),
                ),
            ]),

            // TODO: Force the accessed card to be a scheme
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
    TutorialDisplay::SpeechBubble(SpeechBubble {
        text: text.into(),
        side: Side::Champion,
        delay,
        recurring: false,
    })
}

fn user_say_recurring(text: impl Into<String>, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::SpeechBubble(SpeechBubble {
        text: text.into(),
        side: Side::Champion,
        delay,
        recurring: true,
    })
}

fn opponent_say(text: impl Into<String>, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::SpeechBubble(SpeechBubble {
        text: text.into(),
        side: Side::Overlord,
        delay,
        recurring: false,
    })
}

fn tooltip(text: impl Into<String>, anchor: TooltipAnchor, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::Tooltip(Tooltip { text: text.into(), anchor, delay, recurring: false })
}

fn tooltip_recurring(
    text: impl Into<String>,
    anchor: TooltipAnchor,
    delay: Milliseconds,
) -> TutorialDisplay {
    TutorialDisplay::Tooltip(Tooltip { text: text.into(), anchor, delay, recurring: true })
}

fn toast(text: impl Into<String>) -> TutorialDisplay {
    TutorialDisplay::Toast(Toast {
        text: text.into(),
        delay: Milliseconds(0),
        hide_after: Some(Milliseconds(10_000)),
        recurring: false,
    })
}

fn toast_at(text: impl Into<String>, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::Toast(Toast { text: text.into(), delay, hide_after: None, recurring: false })
}
