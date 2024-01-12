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

use core_data::game_primitives::{Milliseconds, RoomId, Side};
use core_ui::design::{self, FontColor};
use core_ui::icons;
use game_data::card_name::CardName;
use game_data::game_actions::CardTarget;
use game_data::tutorial_data::{
    SpeechBubble, Toast, Tooltip, TooltipAnchor, TutorialDisplay, TutorialGameStateTrigger,
    TutorialMessageKey, TutorialMessageTrigger, TutorialOpponentAction, TutorialSequence,
    TutorialStep, TutorialTrigger,
};
use once_cell::sync::Lazy;

pub mod tutorial_actions;

pub const PLAYER_SIDE: Side = Side::Riftcaller;
pub const OPPONENT_SIDE: Side = Side::Covenant;

/// Definition for the game tutorial
pub static SEQUENCE: Lazy<TutorialSequence> = Lazy::new(|| {
    TutorialSequence {

        // The first few turns of the tutorial game are pre-scripted and
        // defined here
        steps: vec![
            TutorialStep::SetHand(Side::Covenant, vec![CardName::Frog]),
            TutorialStep::SetHand(Side::Riftcaller, vec![CardName::EldritchSurge, CardName::SimpleAxe]),
            TutorialStep::SetTopOfDeck(Side::Covenant, vec![CardName::Captain, CardName::Machinate]),
            TutorialStep::AddGameModifiers(vec![
                CardName::TutorialDisableDrawAction,
                CardName::TutorialDisableGainMana,
                CardName::TutorialDisableRaidSanctum,
                CardName::TutorialDisableRaidVault,
                CardName::TutorialDisableRaidCrypt,
                CardName::TutorialDisableRaidOuter,
                CardName::TutorialDisableRaidContinue,
                CardName::TutorialDisableEndRaid
            ]),
            TutorialStep::KeepOpeningHand(Side::Riftcaller),
            TutorialStep::KeepOpeningHand(Side::Covenant),
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
                permanent_toast(
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
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::PlayAnyCard]),
            TutorialStep::Display(vec![
                user_say("No evil shall stand against my valor.", Milliseconds(4000)),
                permanent_toast(
                    format!(
                        "Playing cards from your hand costs one {} (action point).",
                        icons::ACTION
                    ),
                    Milliseconds(0),
                ),
                user_say("I should play a card...", Milliseconds(20_000)),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::PlayAnyCard]),
            // User -> 4 mana
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableRaidOuter,
            ]),
            TutorialStep::Display(vec![
                user_say("My weapon is ready.", Milliseconds(0)),
                user_say("I should investigate that room...", Milliseconds(4000)),
                permanent_toast(
                    format!("You can spend {} to start a <b>raid</b> and explore a <b>room</b> of the enemy's dungeon.", icons::ACTION),
                    Milliseconds(6000),
                ),
                tooltip(
                    "Drag character here",
                    TooltipAnchor::RaidRoom(RoomId::RoomA),
                    Milliseconds(8000),
                ),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::InitiateRaid(RoomId::RoomA)]),
            TutorialStep::OpponentAction(TutorialOpponentAction::SummonMinion(
                CardName::Captain
            )),
            TutorialStep::Display(vec![
                permanent_toast(
                    "To get past a defending minion, you must deal damage to it equal to its <b>health</b>.",
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::UseWeapon {
                weapon: CardName::SimpleAxe,
                target: CardName::Captain,
            }]),
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableRaidContinue,
            ]),
            TutorialStep::Display(vec![
                permanent_toast(
                    "You have accessed the room and can now <b>score</b> this card for 30 points.",
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::ScoreAccessedCard(
                CardName::Machinate,
            )]),
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableEndRaid,
            ]),
            TutorialStep::Display(vec![
                permanent_toast(
                    "Scoring <b>scheme</b> cards in rooms gives you points. The first player to reach 60 points wins!",
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::Display(vec![opponent_say("Curse you!", Milliseconds(0))]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::SuccessfullyEndRaid]),
            // User -> 4 mana
            TutorialStep::SetTopOfDeck(Side::Covenant, vec![CardName::GatheringDark]),
            TutorialStep::Display(vec![opponent_say("My power grows.", Milliseconds(0))]),
            TutorialStep::OpponentAction(TutorialOpponentAction::GainMana),
            TutorialStep::OpponentAction(TutorialOpponentAction::GainMana),
            TutorialStep::OpponentAction(TutorialOpponentAction::GainMana),
            // Opponent -> 5 mana
            TutorialStep::SetTopOfDeck(
                Side::Riftcaller,
                vec![CardName::Lodestone, CardName::ArcaneRecovery],
            ),
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableGainMana,
            ]),
            TutorialStep::Display(vec![
                user_say("I need more mana...", Milliseconds(0)),
                permanent_toast(
                    format!("You can spend {} to gain 1{}.", icons::ACTION, icons::MANA),
                    Milliseconds(4000),
                ),
                tooltip("Tap to gain mana", TooltipAnchor::GainMana, Milliseconds(4_000)),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::GainManaAction]),
            // User -> 5 mana
            TutorialStep::Display(vec![
                user_say("You'll pay for what you did.", Milliseconds(0)),
                permanent_toast("Now you can play this card", Milliseconds(4000)),
                user_say("I should play a card...", Milliseconds(20_000))
                ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::PlayCard(
                CardName::ArcaneRecovery,
                CardTarget::None,
            )]),
            // User -> 9 mana
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableDrawAction,
            ]),
            TutorialStep::Display(vec![
                user_say("I should draw another card...", Milliseconds(0)),
                permanent_toast(
                    format!("You can spend {} to draw a card.", icons::ACTION),
                    Milliseconds(4000),
                ),
                tooltip("Tap to draw card", TooltipAnchor::DrawCard, Milliseconds(4000)),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::DrawCardAction]),
            TutorialStep::SetTopOfDeck(Side::Covenant, vec![CardName::Devise]),
            TutorialStep::OpponentAction(TutorialOpponentAction::PlayCard(
                CardName::GatheringDark,
                CardTarget::None,
            )),
            TutorialStep::OpponentAction(TutorialOpponentAction::PlayCard(
                CardName::Devise,
                CardTarget::Room(RoomId::RoomA),
            )),
            TutorialStep::Display(vec![opponent_say("Arise, my minions!", Milliseconds(0))]),
            TutorialStep::OpponentAction(TutorialOpponentAction::PlayCard(
                CardName::Frog,
                CardTarget::Room(RoomId::RoomA),
            )),
            TutorialStep::SetTopOfDeck(
                Side::Riftcaller,
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
                permanent_toast(
                    "Minions are never permanently defeated. They respawn between raid_display.",
                    Milliseconds(0),
                ),
                user_say("You can't keep me out of that room.", Milliseconds(4000)),
                user_say_recurring("I should raid again.", Milliseconds(10_000)),
                tooltip_recurring(
                    "Drag character here",
                    TooltipAnchor::RaidRoom(RoomId::RoomA),
                    Milliseconds(15_000),
                ),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::InitiateRaid(RoomId::RoomA)]),
            TutorialStep::OpponentAction(TutorialOpponentAction::SummonMinion(
                CardName::Frog
            )),
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableRaidVault,
            ]),
            TutorialStep::Display(vec![
                user_say("Law and Light! My weapon is useless.", Milliseconds(0)),
                permanent_toast(
                    format!("A <color={}>Mortal</color> weapon cannot damage an <color={}>Abyssal</color> minion. A matching weapon is required!",
                    design::as_hex(FontColor::MortalCardTitle),
                    design::as_hex(FontColor::AstralCardTitle)),
                    Milliseconds(4000),
                ),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::UseNoWeapon]),
            TutorialStep::Display(vec![
                permanent_toast(
                    "You can raid the <b>Vault</b> and attempt to score cards on top of your opponent's deck.",
                    Milliseconds(2000)),
                user_say_recurring("I should search the Vault.", Milliseconds(8_000)),
                tooltip_recurring(
                    "Drag character here",
                    TooltipAnchor::RaidRoom(RoomId::Vault),
                    Milliseconds(10_000),
                ),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::InitiateRaid(RoomId::Vault)]),
            TutorialStep::Display(vec![
                permanent_toast(
                    "If the top card of your opponent's deck is a <b>scheme</b> card, you can score it for points!",
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::SetTopOfDeck(Side::Covenant, vec![CardName::Machinate, CardName::Conspire]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::SuccessfullyEndRaid]),
            TutorialStep::AddGameModifiers(vec![
                CardName::TutorialDisableRaidVault,
            ]),
            TutorialStep::Display(vec![
                permanent_toast(
                    format!("Now you just need to find an <color={}>Abyssal</color> weapon",
                    design::as_hex(FontColor::AstralCardTitle)),
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::Display(vec![
                user_say_recurring("I should draw a card", Milliseconds(10_000)),
            ]),
            TutorialStep::OpponentAction(TutorialOpponentAction::ProgressRoom(
                RoomId::RoomA,
            )),
            TutorialStep::OpponentAction(TutorialOpponentAction::ProgressRoom(
                RoomId::RoomA,
            )),
            TutorialStep::SetTopOfDeck(Side::Riftcaller, vec![CardName::SimpleClub]),
            TutorialStep::OpponentAction(TutorialOpponentAction::ProgressRoom(
                RoomId::RoomA,
            )),
            TutorialStep::Display(vec![
                permanent_toast(
                    format!("Your opponent can <b>progress</b> the cards in a room for {} and 1{}. If the <b>progress requirement</b> of a scheme card is met, they score it.", icons::ACTION, icons::MANA),
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::AwaitGameState(TutorialGameStateTrigger::HandContainsCard(Side::Riftcaller, CardName::SimpleClub)),
            TutorialStep::Display(vec![
                user_say("The weapon I need!", Milliseconds(0)),
                user_say_recurring("I should play this Simple Club", Milliseconds(10_000)),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::PlayCard(CardName::SimpleClub, CardTarget::None)]),
            TutorialStep::Display(vec![
                user_say("Time to try again.", Milliseconds(0)),
                user_say_recurring("I can use this weapon to raid that room.", Milliseconds(10_000)),
                tooltip_recurring(
                    "Drag character here",
                    TooltipAnchor::RaidRoom(RoomId::RoomA),
                    Milliseconds(15_000),
                ),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::InitiateRaid(RoomId::RoomA)]),
            TutorialStep::Display(vec![
                permanent_toast(
                    format!("During a raid encounter, you can power up your weapon's damage using {}", icons::MANA),
                    Milliseconds(0),
                ),
            ]),
            TutorialStep::RemoveGameModifiers(vec![
                CardName::TutorialDisableRaidSanctum,
            ]),
            TutorialStep::AddGameModifiers(vec![
                CardName::TutorialForceSanctumScore,
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::SuccessfullyEndRaid]),
            TutorialStep::Display(vec![
                user_say_recurring("Time to end this.", Milliseconds(4_000)),
                permanent_toast(
                    "You can raid the <b>Sanctum</b> to access a random card from your opponent's hand",
                    Milliseconds(2000)),
                tooltip_recurring(
                    "Drag character here",
                    TooltipAnchor::RaidRoom(RoomId::Sanctum),
                    Milliseconds(8_000),
                ),
            ]),
            TutorialStep::AwaitTriggers(vec![TutorialTrigger::InitiateRaid(RoomId::Sanctum)]),
            TutorialStep::Display(vec![
                permanent_toast(
                    "If the random card in hand is a <b>scheme</b> card, you can score it!",
                    Milliseconds(0)
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
                        design::as_hex(FontColor::AstralCardTitle)),
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
        side: Side::Riftcaller,
        delay,
        recurring: false,
    })
}

fn user_say_recurring(text: impl Into<String>, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::SpeechBubble(SpeechBubble {
        text: text.into(),
        side: Side::Riftcaller,
        delay,
        recurring: true,
    })
}

fn opponent_say(text: impl Into<String>, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::SpeechBubble(SpeechBubble {
        text: text.into(),
        side: Side::Covenant,
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
    })
}

fn permanent_toast(text: impl Into<String>, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::Toast(Toast { text: text.into(), delay, hide_after: None })
}
