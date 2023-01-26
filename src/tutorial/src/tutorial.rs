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

use data::card_name::CardName;
use data::game_actions::CardTarget;
use data::primitives::{Milliseconds, RoomId, Side};
use data::tutorial_data::{
    SpeechBubble, Tooltip, TooltipAnchor, TutorialAction, TutorialDisplay, TutorialStep,
};
use once_cell::sync::Lazy;

pub const PLAYER_SIDE: Side = Side::Champion;
pub const OPPONENT_SIDE: Side = Side::Overlord;

/// Sequence describing the events of the game's tutorial
pub static STEPS: Lazy<Vec<TutorialStep>> = Lazy::new(|| {
    vec![
        TutorialStep::SetHand(Side::Overlord, vec![CardName::Frog]),
        TutorialStep::SetHand(Side::Champion, vec![CardName::EldritchSurge]),
        TutorialStep::SetTopOfDeck(Side::Overlord, vec![CardName::Scout, CardName::Machinate]),
        TutorialStep::SetTopOfDeck(Side::Champion, vec![CardName::SimpleAxe]),
        TutorialStep::KeepOpeningHand(Side::Champion),
        TutorialStep::KeepOpeningHand(Side::Overlord),
        TutorialStep::OpponentAction(TutorialAction::DrawCard),
        TutorialStep::OpponentAction(TutorialAction::PlayCard(
            CardName::Machinate,
            CardTarget::Room(RoomId::RoomA),
        )),
        TutorialStep::OpponentAction(TutorialAction::PlayCard(
            CardName::Scout,
            CardTarget::Room(RoomId::RoomA),
        )),
        TutorialStep::Display(vec![
            opponent_say("Surrender to the night!", Milliseconds(2000)),
            user_say("I should play a card...", Milliseconds(10_000)),
        ]),
        TutorialStep::AwaitPlayerActions(vec![
            TutorialAction::PlayCard(CardName::EldritchSurge, CardTarget::None),
            TutorialAction::PlayCard(CardName::SimpleAxe, CardTarget::None),
        ]),
        TutorialStep::Display(vec![
            user_say("No evil shall stand against my valor!", Milliseconds(0)),
            user_say("I should investigate that room...", Milliseconds(5000)),
            tooltip(
                "Drag portrait here",
                TooltipAnchor::RaidRoom(RoomId::RoomA),
                Milliseconds(5000),
            ),
        ]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::InitiateRaid(RoomId::RoomA)]),
        TutorialStep::Display(vec![user_say("For honor!", Milliseconds(0))]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::UseWeapon {
            weapon: CardName::SimpleAxe,
            target: CardName::Scout,
        }]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::ScoreAccessedCard(
            CardName::Machinate,
        )]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::EndRaid]),
        TutorialStep::SetTopOfDeck(Side::Overlord, vec![CardName::GatheringDark]),
        TutorialStep::OpponentAction(TutorialAction::GainMana),
        TutorialStep::OpponentAction(TutorialAction::GainMana),
        TutorialStep::OpponentAction(TutorialAction::GainMana),
        TutorialStep::SetTopOfDeck(
            Side::Champion,
            vec![CardName::ArcaneRecovery, CardName::Lodestone],
        ),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::GainMana]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::PlayCard(
            CardName::ArcaneRecovery,
            CardTarget::None,
        )]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::DrawCard]),
        TutorialStep::SetTopOfDeck(Side::Overlord, vec![CardName::Devise]),
        TutorialStep::OpponentAction(TutorialAction::PlayCard(
            CardName::GatheringDark,
            CardTarget::None,
        )),
        TutorialStep::OpponentAction(TutorialAction::PlayCard(
            CardName::Devise,
            CardTarget::Room(RoomId::RoomA),
        )),
        TutorialStep::OpponentAction(TutorialAction::PlayCard(
            CardName::Frog,
            CardTarget::Room(RoomId::RoomA),
        )),
        TutorialStep::SetTopOfDeck(
            Side::Champion,
            vec![CardName::SimpleHammer, CardName::Contemplate, CardName::SimpleClub],
        ),
    ]
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
