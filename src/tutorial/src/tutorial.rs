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

use core_ui::icons;
use data::card_name::CardName;
use data::game_actions::CardTarget;
use data::primitives::{Milliseconds, RoomId, Side};
use data::tutorial_data::{
    SpeechBubble, Toast, Tooltip, TooltipAnchor, TutorialAction, TutorialDisplay, TutorialStep,
};
use once_cell::sync::Lazy;

pub const PLAYER_SIDE: Side = Side::Champion;
pub const OPPONENT_SIDE: Side = Side::Overlord;

/// Sequence describing the events of the game's tutorial
pub static STEPS: Lazy<Vec<TutorialStep>> = Lazy::new(|| {
    vec![
        TutorialStep::SetHand(Side::Overlord, vec![CardName::Frog]),
        TutorialStep::SetHand(Side::Champion, vec![CardName::EldritchSurge]),
        TutorialStep::SetTopOfDeck(Side::Overlord, vec![CardName::Captain, CardName::Machinate]),
        TutorialStep::SetTopOfDeck(Side::Champion, vec![CardName::SimpleAxe]),
        TutorialStep::KeepOpeningHand(Side::Champion),
        TutorialStep::KeepOpeningHand(Side::Overlord),
        TutorialStep::OpponentAction(TutorialAction::DrawCard),
        TutorialStep::OpponentAction(TutorialAction::PlayCard(
            CardName::Machinate,
            CardTarget::Room(RoomId::RoomA),
        )),
        TutorialStep::OpponentAction(TutorialAction::PlayCard(
            CardName::Captain,
            CardTarget::Room(RoomId::RoomA),
        )),
        TutorialStep::Display(vec![
            toast(
                format!(
                    "<b>Mana</b> ({}) lets you play cards and use weapons. It persists between turns.",
                    icons::MANA
                ),
                Milliseconds(0),
            ),
            opponent_say("Surrender to the night!", Milliseconds(4000)),
            user_say("Your tyranny ends here, Vaughn!", Milliseconds(8000)),
            user_say("I should play a card...", Milliseconds(20_000)),
        ]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::PlayAnyCard]),
        TutorialStep::Display(vec![
            user_say("No evil shall stand against my valor.", Milliseconds(4000)),
            toast(
                format!(
                    "Playing cards from your hand costs one {} (<b>action point</b>).",
                    icons::ACTION
                ),
                Milliseconds(0),
            ),
            user_say("I should play a card...", Milliseconds(20_000)),
        ]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::PlayAnyCard]),
        // User -> 4 mana
        TutorialStep::Display(vec![
            toast(
                "Weapons let you deal <b>damage</b> to defeat enemy minions.",
                Milliseconds(0),
            ),
            user_say("My weapon is ready.", Milliseconds(4000)),
            user_say("I should investigate that room...", Milliseconds(8000)),
            tooltip(
                "Drag portrait here",
                TooltipAnchor::RaidRoom(RoomId::RoomA),
                Milliseconds(10_000),
            ),
            toast(
                format!("You can spend {} to initiate a <b>raid</b>.", icons::ACTION),
                Milliseconds(10_000),
            ),
        ]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::InitiateRaid(RoomId::RoomA)]),
        TutorialStep::Display(vec![
            toast(
                "You have started a <b>raid</b>, which will let you explore one the <b>rooms</b> of the enemy's dungeon.",
                Milliseconds(0),
            ),
            toast(
                "You can use your new weapon to defeat this minion.",
                Milliseconds(8000),
            ),
        ]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::UseWeapon {
            weapon: CardName::SimpleAxe,
            target: CardName::Captain,
        }]),
        TutorialStep::Display(vec![
            toast(
                "To get past a defending minion, you must deal damage to it equal to its <b>health</b>.",
                Milliseconds(0),
            ),
            toast(
                "Once you access a room, you can <b>score</b> a card inside.",
                Milliseconds(8000),
            ),
        ]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::ScoreAccessedCard(
            CardName::Machinate,
        )]),
        TutorialStep::Display(vec![
            toast(
                "Scoring <b>scheme</b> cards in rooms gives you points. The first player to reach 100 points wins!",
                Milliseconds(0),
            ),
        ]),
        TutorialStep::Display(vec![opponent_say("Curse you!", Milliseconds(0))]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::EndRaid]),
        // User -> 4 mana
        TutorialStep::SetTopOfDeck(Side::Overlord, vec![CardName::GatheringDark]),
        TutorialStep::OpponentAction(TutorialAction::GainMana),
        TutorialStep::OpponentAction(TutorialAction::GainMana),
        TutorialStep::OpponentAction(TutorialAction::GainMana),
        // Opponent -> 5 mana
        TutorialStep::SetTopOfDeck(
            Side::Champion,
            vec![CardName::ArcaneRecovery, CardName::Lodestone],
        ),
        TutorialStep::Display(vec![
            toast(
                format!("You can spend {} to gain 1{}.", icons::ACTION, icons::MANA),
                Milliseconds(0),
            ),
            user_say("You'll pay for what you did.", Milliseconds(4000)),
            user_say("I need more mana...", Milliseconds(8000)),
            tooltip("Tap to gain mana", TooltipAnchor::GainMana, Milliseconds(11_000)),
        ]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::GainMana]),
        // User -> 5 mana
        TutorialStep::Display(vec![user_say("I should play a card...", Milliseconds(20_000))]),
        TutorialStep::AwaitPlayerActions(vec![TutorialAction::PlayCard(
            CardName::ArcaneRecovery,
            CardTarget::None,
        )]),
        // User -> 9 mana
        TutorialStep::Display(vec![
            toast(
                format!("You can also spend {} to draw a card.", icons::ACTION),
                Milliseconds(0),
            ),
            user_say("I should draw another card...", Milliseconds(4000)),
            tooltip("Tap to draw card", TooltipAnchor::DrawCard, Milliseconds(7000)),
        ]),
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

fn toast(text: impl Into<String>, delay: Milliseconds) -> TutorialDisplay {
    TutorialDisplay::Toast(Toast { text: text.into(), delay })
}
