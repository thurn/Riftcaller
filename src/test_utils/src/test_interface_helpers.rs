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

use anyhow::Result;
use core_data::game_primitives::Side;
use core_ui::icons;
use game_data::card_name::CardName;
use game_data::player_name::PlayerId;
use protos::riftcaller::client_action::Action;
use protos::riftcaller::{CardIdentifier, EventHandlers};
use server::server_data::GameResponseOutput;
use with_error::{verify, WithError};

use crate::client_interface::HasText;
use crate::test_session::TestSession;
use crate::TestSessionHelpers;

#[derive(Debug)]
pub enum Button {
    Summon,
    NoSummon,
    NoWeapon,
    ProceedToAccess,
    Score,
    EndRaid,
    EndTurn,
    SubmitDiscard,
    SubmitCardSelector,
    StartTurn,
    Sacrifice,
    NoPromptAction,
    Destroy,
    Discard,
    CancelPlayingCard,
    SkipPlayingCard,
    SkipSelectingRoom,
    InitiateRaid,
    DraftPick,
    ShowDeck,
    CloseIcon,
    StartBattle,
    ReturnToDeck,
    ReturnToHand,
    SelectForMultipart,
    SwapCard,
    ChooseOnPlay,
    ChooseForPrompt,
    ChooseOccupantForPrompt,
    ChooseDefenderForPrompt,
    Evade,
    ClosePriorityPrompt,
    AccessAnother,
    EndAccess,
    Play,
    Reveal,
    Pay,
    ChooseCardTypeSpell,
    Prevent,
    AddPowerCharges,
    AccessVault,
    AccessSanctum,
    DefeatMinion,
}

pub trait TestInterfaceHelpers {
    /// Look for a button in the user interface and invoke its action as the
    /// current user.
    fn click(&mut self, button: Button) -> GameResponseOutput;

    fn click_with_result(&mut self, button: Button) -> Result<GameResponseOutput>;

    /// Click a button attached to a given card
    fn click_card_button(&mut self, player_id: PlayerId, card_id: CardIdentifier, button: Button);

    fn click_card_button_with_result(
        &mut self,
        player_id: PlayerId,
        card_id: CardIdentifier,
        button: Button,
    ) -> Result<GameResponseOutput>;

    /// Look for a button in the user interface and invoke its action as the
    /// opponent of the current user.
    fn opponent_click(&mut self, button: Button) -> GameResponseOutput;

    /// Clicks on a button in the user interface as the `side` player.
    fn click_as_side(&mut self, button: Button, side: Side) -> GameResponseOutput;

    /// Returns true if the matching button can be found anywhere in the user
    /// interface for the current user.
    fn has(&self, button: Button) -> bool;

    /// Returns true if the matching button can be found anywhere in the user
    /// interface for the `side` user.
    fn side_has(&self, button: Button, side: Side) -> bool;

    /// Returns true if any UI for the user contains the given card's name
    fn has_card_name(&self, card_name: CardName) -> bool;

    /// Clicks as the user on a button as the showing a given card's name
    fn click_card_name(&mut self, card_name: CardName);

    /// Equivalent function to `click_card_name` which returns a Result.
    fn click_card_name_with_result(&mut self, card_name: CardName) -> Result<GameResponseOutput>;

    /// Locate a button containing the provided `text` in the provided player's
    /// interface controls and invoke its registered action.
    fn click_on(&mut self, player_id: PlayerId, text: impl Into<String>) -> GameResponseOutput;

    fn click_on_with_result(
        &mut self,
        player_id: PlayerId,
        text: impl Into<String>,
    ) -> Result<GameResponseOutput>;

    /// Returns true if the provided text can be found anywhere in the user
    /// interface.
    fn has_text(&self, text: impl Into<String>) -> bool;

    /// Returns the number of panels which are currently open
    fn open_panel_count(&self) -> usize;
}

impl TestInterfaceHelpers for TestSession {
    fn click(&mut self, button: Button) -> GameResponseOutput {
        let text = resolve_button(button);
        self.click_on(self.user_id(), text)
    }

    fn click_with_result(&mut self, button: Button) -> Result<GameResponseOutput> {
        let text = resolve_button(button);
        self.click_on_with_result(self.user_id(), text)
    }

    fn click_card_button(&mut self, player_id: PlayerId, card_id: CardIdentifier, button: Button) {
        self.click_card_button_with_result(player_id, card_id, button)
            .expect("Error clicking card button");
    }

    fn click_card_button_with_result(
        &mut self,
        player_id: PlayerId,
        card_id: CardIdentifier,
        button: Button,
    ) -> Result<GameResponseOutput> {
        let player = self.player(player_id);
        let node = player
            .interface
            .card_anchors()
            .iter()
            .find(|card_anchor| card_anchor.card_id == Some(card_id))
            .as_ref()
            .unwrap_or_else(|| panic!("Button {button:?} not found for card {card_id:?}"))
            .node
            .as_ref()
            .unwrap_or_else(|| panic!("Node button {button:?} not found for card {card_id:?}"));

        let text = resolve_button(button);
        let handlers = node.find_handlers(text);
        invoke_click(self, player_id, handlers)
    }

    fn opponent_click(&mut self, button: Button) -> GameResponseOutput {
        let text = resolve_button(button);
        self.click_on(self.opponent_id(), text)
    }

    fn click_as_side(&mut self, button: Button, side: Side) -> GameResponseOutput {
        let id = self.player_id_for_side(side);
        if id == self.user_id() {
            self.click(button)
        } else {
            self.opponent_click(button)
        }
    }

    fn has(&self, button: Button) -> bool {
        let text = resolve_button(button);
        self.client.interface.all_active_nodes().has_text(text)
    }

    fn side_has(&self, button: Button, side: Side) -> bool {
        let id = self.player_id_for_side(side);
        let text = resolve_button(button);
        if id == self.user_id() {
            self.client.interface.all_active_nodes().has_text(text)
        } else {
            self.opponent.interface.all_active_nodes().has_text(text)
        }
    }

    fn has_card_name(&self, card_name: CardName) -> bool {
        self.has_text(card_name.displayed_name())
    }

    fn click_card_name(&mut self, card_name: CardName) {
        self.click_on(self.player_id_for_side(Side::Riftcaller), card_name.displayed_name());
    }

    fn click_card_name_with_result(&mut self, card_name: CardName) -> Result<GameResponseOutput> {
        self.click_on_with_result(
            self.player_id_for_side(Side::Riftcaller),
            card_name.displayed_name(),
        )
    }

    fn click_on(&mut self, player_id: PlayerId, text: impl Into<String>) -> GameResponseOutput {
        let string = text.into();
        self.click_on_with_result(player_id, string.clone()).unwrap_or_else(|e| {
            let t = self.player(player_id).interface.all_active_nodes().all_text();
            panic!("Error clicking on {string}.\nCurrent Text:\n{t}\nError: {e:?}")
        })
    }

    fn click_on_with_result(
        &mut self,
        player_id: PlayerId,
        text: impl Into<String>,
    ) -> Result<GameResponseOutput> {
        let player = self.player(player_id);
        let handlers = player.interface.all_active_nodes().find_handlers(text);
        invoke_click(self, player_id, handlers)
    }

    fn has_text(&self, text: impl Into<String>) -> bool {
        self.client.interface.all_active_nodes().has_text(text.into())
    }

    fn open_panel_count(&self) -> usize {
        self.client.interface.panel_count()
    }
}

fn invoke_click(
    session: &mut TestSession,
    player_id: PlayerId,
    handlers: Option<EventHandlers>,
) -> Result<GameResponseOutput> {
    let action = handlers
        .with_error(|| "Button not found")?
        .on_click
        .with_error(|| "OnClick not found")?
        .action
        .with_error(|| "Action not found")?;
    if let Action::StandardAction(a) = &action {
        verify!(!(a.payload.is_empty() && a.update.is_none()), "Attempted to invoke empty action");
    }
    session.perform_action(action, player_id)
}

fn resolve_button(button: Button) -> String {
    match button {
        Button::Summon => "Summon",
        Button::NoSummon => "Pass",
        Button::NoWeapon => "Continue",
        Button::ProceedToAccess => "Proceed",
        Button::Score => "Score",
        Button::EndRaid => "End Raid",
        Button::EndTurn => "End Turn",
        Button::SubmitDiscard => "Submit",
        Button::SubmitCardSelector => "Submit",
        Button::StartTurn => "Start Turn",
        Button::Sacrifice => "Sacrifice",
        Button::NoPromptAction => "Continue",
        Button::Destroy => "Destroy",
        Button::Discard => "Discard",
        Button::CancelPlayingCard => "Cancel",
        Button::SkipPlayingCard => "Skip",
        Button::SkipSelectingRoom => "Skip",
        Button::InitiateRaid => "Initiate Raid",
        Button::DraftPick => "Pick",
        Button::ShowDeck => icons::DECK,
        Button::CloseIcon => icons::CLOSE,
        Button::StartBattle => "Start",
        Button::ReturnToDeck => "Return",
        Button::ReturnToHand => "Return",
        Button::SelectForMultipart => "Select",
        Button::SwapCard => "Swap",
        Button::ChooseOnPlay => "Choose",
        Button::ChooseForPrompt => "Choose",
        Button::ChooseOccupantForPrompt => "Occupant",
        Button::ChooseDefenderForPrompt => "Defender",
        Button::Evade => "Evade",
        Button::ClosePriorityPrompt => "Continue",
        Button::AccessAnother => "Access Another",
        Button::EndAccess => "End Access",
        Button::Play => "Play",
        Button::Reveal => "Reveal",
        Button::Pay => "Pay",
        Button::ChooseCardTypeSpell => "Spell",
        Button::Prevent => "Prevent",
        Button::AddPowerCharges => "Add",
        Button::AccessVault => "Access Vault",
        Button::AccessSanctum => "Access Sanctum",
        Button::DefeatMinion => "Defeat",
    }
    .to_string()
}
