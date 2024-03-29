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

use adapters::CustomCardIdentifier;
use anyhow::Result;
use card_definition_data::cards;
use core_data::game_primitives::{CardType, Resonance, RoomId, Side};
use game_data::card_name::{CardName, CardVariant};
use game_data::player_name::PlayerId;
use protos::riftcaller::client_action::Action;
use protos::riftcaller::{
    card_target, CardIdentifier, CardTarget, DrawCardAction, GainManaAction, GameMessageType,
    InitiateRaidAction, MoveCardAction, PlayCardAction, ProgressRoomAction, SpendActionPointAction,
};
use server::server_data::GameResponseOutput;

use crate::test_game_client::{ClientPlayer, TestGameClient};
use crate::test_session::TestSession;
use crate::{test_game_client, Button, CardNamesExt, TestInterfaceHelpers};

pub trait TestSessionHelpers {
    fn user_id(&self) -> PlayerId;

    fn opponent_id(&self) -> PlayerId;

    /// Returns the [TestGameClient] for a given player in the game.
    fn player(&self, player_id: PlayerId) -> &TestGameClient;

    /// Returns the [TestGameClient] for the [Side] player in the game.
    fn player_for_side(&self, side: Side) -> &TestGameClient;

    /// Returns the user player state for the user client, (i.e. the user's
    /// state from *their own* perspective).
    fn me(&self) -> &ClientPlayer;

    /// Returns the opponent player state for the opponent client (i.e. the
    /// opponent's state from their perspective).
    fn you(&self) -> &ClientPlayer;

    /// Equivalent function to [TestSession::perform_action] which does not
    /// return the action result.
    fn perform(&mut self, action: Action, user_id: PlayerId);

    /// Helper to perform the standard draw card action
    fn draw_card(&mut self);

    /// Equivalent function to [Self::draw_card] which returns the result
    fn draw_card_with_result(&mut self) -> Result<GameResponseOutput>;

    /// Causes the opponent player to take the draw_card action
    fn opponent_draw_card(&mut self);

    /// Helper to perform the standard gain mana action
    fn gain_mana(&mut self);

    /// Equivalent function to [Self::gain_mana] which returns the result
    fn gain_mana_with_result(&mut self) -> Result<GameResponseOutput>;

    /// Helper function to perform an action to initiate a raid on the provided
    /// `room_id`.
    fn initiate_raid(&mut self, room_id: RoomId) -> GameResponseOutput;

    /// Helper function to perform an action to progress the
    /// provided `room_id`.
    fn progress_room(&mut self, room_id: RoomId) -> GameResponseOutput;

    /// Progresses the [test_constants::CLIENT_ROOM_ID] room a specified number
    /// of `times`. If this requires multiple turns, spends the Riftcaller turns
    /// doing nothing.
    fn progress_room_times(&mut self, times: u32);

    /// Helper to take the [PlayCardAction] with a given card ID.
    fn play_card(&mut self, card_id: CardIdentifier, player_id: PlayerId, target: Option<RoomId>);

    /// Equivalent function to [Self::play_card] which returns a result.
    fn play_card_with_result(
        &mut self,
        card_id: CardIdentifier,
        player_id: PlayerId,
        target: Option<RoomId>,
    ) -> Result<GameResponseOutput>;

    /// Creates and then plays a named card as the user who owns this card.
    ///
    /// This function first adds a copy of the requested card to the user's hand
    /// via [TestSession::add_to_hand]. The card is then played via the standard
    /// [PlayCardAction]. Action points and mana must be available and are spent
    /// as normal.
    ///
    /// If the card is a minion, project, or scheme card, it is played
    /// into the [test_constants::ROOM_ID] room. The [CardIdentifier] for the
    /// played card is returned.
    ///
    /// Panics if the server returns an error for playing this card.
    fn create_and_play(&mut self, card_name: CardName) -> CardIdentifier;

    /// Equivalent method to [Self::create_and_play] which creates the upgraded
    /// version of the card.
    fn create_and_play_upgraded(&mut self, card_name: CardName) -> CardIdentifier;

    /// Equivalent method to [Self::create_and_play] which specifies
    /// a target room to use.
    fn create_and_play_with_target(
        &mut self,
        card_name: CardName,
        room_id: RoomId,
    ) -> CardIdentifier;

    /// Equivalent method to [Self::create_and_play] which creates the upgraded
    /// version of the card.
    fn create_and_play_upgraded_with_target(
        &mut self,
        card_name: CardName,
        room_id: RoomId,
    ) -> CardIdentifier;

    /// Activates an ability of a card owned by the user based on its ability
    /// index.
    fn activate_ability(&mut self, card_id: CardIdentifier, index: u32);

    /// Equivalent to [Self::activate_ability] which returns the result.
    fn activate_ability_with_result(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
    ) -> Result<GameResponseOutput>;

    /// Activates an ability of a card with a target room
    fn activate_ability_with_target(&mut self, card_id: CardIdentifier, index: u32, target: RoomId);

    /// Equivalent to [Self::activate_ability] which returns the result.
    fn activate_ability_with_target_and_result(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
        target: RoomId,
    ) -> Result<GameResponseOutput>;

    /// Activate an ability of one of the opponent's cards
    fn opponent_activate_ability(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
        target: Option<RoomId>,
    ) -> Result<GameResponseOutput>;

    /// Looks for a card named "curse" in the Riftcaller's hand and plays it.
    fn remove_curse(&mut self);

    /// Summons a project card in play, paying its mana cost and turning it face
    /// up.
    fn summon_project(&mut self, card_id: CardIdentifier);

    /// Equivalent function to [Self::summon_project] which returns the result.
    fn summon_project_with_result(&mut self, card_id: CardIdentifier)
        -> Result<GameResponseOutput>;

    /// Spends one of the `side` player's action points with no effect
    fn spend_action_point(&mut self, side: Side);

    /// Spends all of the `side` player's action points with no effect
    fn spend_all_action_points(&mut self, side: Side);

    /// Performs the move card action, selecting a card to e.g. be discarded.
    fn move_selector_card(&mut self, card_id: CardIdentifier);

    /// Equivalent function to [Self::move_selector_card] which returns the
    /// result
    fn move_card_with_result(&mut self, card_id: CardIdentifier) -> Result<GameResponseOutput>;

    /// Moves a card to a given index position during a 'card reordering' flow
    /// (e.g. arrange the top cards of your deck in any order)
    fn move_card_to_index(&mut self, card_id: CardIdentifier, index: u32);

    /// Equivalent function to [Self::move_card_to_index] which returns the
    /// result
    fn move_card_to_index_with_result(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
    ) -> Result<GameResponseOutput>;

    /// Spends the `side` player's action points with no effect until they have
    /// no action points remaining and then clicks the "End Turn" button.
    fn move_to_end_step(&mut self, side: Side);

    /// Spends the `side` player's action points with no effect until they have
    /// no action points remaining, clicks on "End Turn", and then starts the
    /// next player's turn (clicking the "Start Turn" button if it appears).
    fn pass_turn(&mut self, side: Side);

    /// Returns true if the last-received Game Message was 'Dawn'.
    fn dawn(&self) -> bool;

    /// Returns true if the last-received Game Message was 'Dusk'.
    fn dusk(&self) -> bool;

    /// Returns true if the last-received Game Messages indicated the `winner`
    /// player won the game
    fn is_victory_for_player(&self, winner: Side) -> bool;

    /// Must be invoked during the Covenant turn. Performs the following
    /// actions:
    /// - Plays a test Scheme card
    /// - Ends the Covenant turn
    /// - Initiates a raid on the [test_constants::ROOM_ID] room
    /// - Summons the minion in the room
    fn set_up_minion_combat(&mut self);

    /// Equivalent to [Self::set_up_minion_combat] which invokes an `action`
    /// function at the start of the Riftcaller's turn.
    fn set_up_minion_combat_with_action(&mut self, action: impl FnOnce(&mut TestSession));

    /// Must be invoked during the Riftcaller turn. Performs the following
    /// actions:
    ///
    /// - Ends the Riftcaller turn
    /// - Plays a 3-1 scheme in the [test_constants::ROOM_ID] room.
    /// - Plays the provided `card_name` minion into that room.
    /// - Ends the Covenant turn.
    ///
    /// Returns a tuple of (scheme_id, minion_id).
    ///
    /// WARNING: This causes the Covenant player to draw for their turn.
    fn setup_raid_target(&mut self, card_name: CardName) -> (CardIdentifier, CardIdentifier);

    /// Must be invoked during the Riftcaller turn. Performs the following
    /// actions:
    ///
    /// - Performs all actions described in [Self::setup_raid_target], creating
    ///   a minion of the indicated [Resonance] with `MINION_HEALTH` health.
    /// - Initiates a raid on the [test_constants::ROOM_ID] room.
    /// - Summons the minion
    /// - Clicks on the button with text matching `name` in order to fire weapon
    ///   abilities.
    ///
    /// WARNING: This causes the Covenant play to draw for their turn.
    fn fire_weapon_combat_abilities(&mut self, resonance: Resonance, name: CardName);
}

impl TestSessionHelpers for TestSession {
    fn user_id(&self) -> PlayerId {
        self.client.id
    }

    fn opponent_id(&self) -> PlayerId {
        self.opponent.id
    }

    fn player(&self, player_id: PlayerId) -> &TestGameClient {
        match () {
            _ if player_id == self.client.id => &self.client,
            _ if player_id == self.opponent.id => &self.opponent,
            _ => panic!("Unknown player id: {player_id:?}"),
        }
    }

    fn player_for_side(&self, side: Side) -> &TestGameClient {
        self.player(self.player_id_for_side(side))
    }

    fn me(&self) -> &ClientPlayer {
        &self.client.this_player
    }

    fn you(&self) -> &ClientPlayer {
        &self.opponent.this_player
    }

    fn perform(&mut self, action: Action, user_id: PlayerId) {
        self.perform_action(action, user_id).expect("Request failed");
    }

    fn draw_card(&mut self) {
        self.draw_card_with_result().expect("Error performing draw card action");
    }

    fn draw_card_with_result(&mut self) -> Result<GameResponseOutput> {
        self.perform_action(Action::DrawCard(DrawCardAction {}), self.user_id())
    }

    fn opponent_draw_card(&mut self) {
        self.perform_action(Action::DrawCard(DrawCardAction {}), self.opponent_id())
            .expect("Error performing opponent_draw_card action");
    }

    fn gain_mana(&mut self) {
        self.gain_mana_with_result().expect("Error performing gain mana action");
    }

    fn gain_mana_with_result(&mut self) -> Result<GameResponseOutput> {
        self.perform_action(Action::GainMana(GainManaAction {}), self.user_id())
    }

    fn initiate_raid(&mut self, room_id: RoomId) -> GameResponseOutput {
        self.perform_action(
            Action::InitiateRaid(InitiateRaidAction {
                room_id: adapters::room_identifier(room_id),
            }),
            self.player_id_for_side(Side::Riftcaller),
        )
        .expect("Server Error")
    }

    fn progress_room(&mut self, room_id: RoomId) -> GameResponseOutput {
        self.perform_action(
            Action::ProgressRoom(ProgressRoomAction {
                room_id: adapters::room_identifier(room_id),
            }),
            self.player_id_for_side(Side::Covenant),
        )
        .expect("Server Error")
    }

    fn progress_room_times(&mut self, times: u32) {
        let mut levels = 0;
        let covenant_id = self.player_id_for_side(Side::Covenant);

        loop {
            while self.player(covenant_id).this_player.actions() > 0 {
                self.perform(
                    Action::ProgressRoom(ProgressRoomAction {
                        room_id: test_constants::CLIENT_ROOM_ID.into(),
                    }),
                    covenant_id,
                );
                levels += 1;

                if levels == times {
                    return;
                }
            }

            self.pass_turn(Side::Covenant);
            assert!(self.dawn());
            self.pass_turn(Side::Riftcaller);
            assert!(self.dusk());
        }
    }

    fn play_card(&mut self, card_id: CardIdentifier, player_id: PlayerId, target: Option<RoomId>) {
        self.perform(
            Action::PlayCard(PlayCardAction {
                card_id: Some(card_id),
                target: target.map(|room_id| CardTarget {
                    card_target: Some(card_target::CardTarget::RoomId(adapters::room_identifier(
                        room_id,
                    ))),
                }),
            }),
            player_id,
        );
    }

    fn play_card_with_result(
        &mut self,
        card_id: CardIdentifier,
        player_id: PlayerId,
        target: Option<RoomId>,
    ) -> Result<GameResponseOutput> {
        self.perform_action(
            Action::PlayCard(PlayCardAction {
                card_id: Some(card_id),
                target: target.map(|room_id| CardTarget {
                    card_target: Some(card_target::CardTarget::RoomId(adapters::room_identifier(
                        room_id,
                    ))),
                }),
            }),
            player_id,
        )
    }

    fn create_and_play(&mut self, card_name: CardName) -> CardIdentifier {
        play_impl(
            self,
            CardVariant::standard(card_name),
            match cards::get(CardVariant::standard(card_name)).card_type {
                CardType::Minion | CardType::Project | CardType::Scheme => {
                    Some(test_constants::ROOM_ID)
                }
                _ => None,
            },
        )
    }

    fn create_and_play_upgraded(&mut self, card_name: CardName) -> CardIdentifier {
        play_impl(
            self,
            CardVariant::upgraded(card_name),
            match cards::get(CardVariant::upgraded(card_name)).card_type {
                CardType::Minion | CardType::Project | CardType::Scheme => {
                    Some(test_constants::ROOM_ID)
                }
                _ => None,
            },
        )
    }

    fn create_and_play_with_target(
        &mut self,
        card_name: CardName,
        room_id: RoomId,
    ) -> CardIdentifier {
        play_impl(self, CardVariant::standard(card_name), Some(room_id))
    }

    fn create_and_play_upgraded_with_target(
        &mut self,
        card_name: CardName,
        room_id: RoomId,
    ) -> CardIdentifier {
        play_impl(self, CardVariant::upgraded(card_name), Some(room_id))
    }

    fn activate_ability(&mut self, card_id: CardIdentifier, index: u32) {
        activate_ability_impl(self, card_id, index, None, self.user_id())
            .expect("Error activating ability");
    }

    fn activate_ability_with_result(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
    ) -> Result<GameResponseOutput> {
        activate_ability_impl(self, card_id, index, None, self.user_id())
    }

    fn activate_ability_with_target(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
        target: RoomId,
    ) {
        activate_ability_impl(self, card_id, index, Some(target), self.user_id())
            .expect("Error activating ability");
    }

    fn activate_ability_with_target_and_result(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
        target: RoomId,
    ) -> Result<GameResponseOutput> {
        activate_ability_impl(self, card_id, index, Some(target), self.user_id())
    }

    fn opponent_activate_ability(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
        target: Option<RoomId>,
    ) -> Result<GameResponseOutput> {
        activate_ability_impl(self, card_id, index, target, self.opponent_id())
    }

    fn remove_curse(&mut self) {
        let player_id = self.player_id_for_side(Side::Riftcaller);
        let id = self
            .player_for_side(Side::Riftcaller)
            .cards
            .hand()
            .token_cards()
            .iter()
            .find(|c| c.title() == "Curse")
            .map(|c| c.id())
            .expect("Curse not found");
        self.play_card(id, player_id, None);
    }

    fn summon_project(&mut self, card_id: CardIdentifier) {
        self.summon_project_with_result(card_id).expect("Error summoning project");
    }

    fn summon_project_with_result(
        &mut self,
        card_id: CardIdentifier,
    ) -> Result<GameResponseOutput> {
        let id = CardIdentifier {
            game_action: Some(CustomCardIdentifier::SummonProject as u32),
            ..card_id
        };
        self.perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(id), target: None }),
            self.player_id_for_side(adapters::side(id.side).expect("Invalid Side")),
        )
    }

    fn spend_action_point(&mut self, side: Side) {
        let id = self.player_id_for_side(side);
        self.perform(Action::SpendActionPoint(SpendActionPointAction {}), id);
    }

    fn spend_all_action_points(&mut self, side: Side) {
        let id = self.player_id_for_side(side);
        while self.player(id).this_player.actions() > 0 {
            self.spend_action_point(side);
        }
    }

    fn move_selector_card(&mut self, card_id: CardIdentifier) {
        self.move_card_with_result(card_id)
            .unwrap_or_else(|_| panic!("Error moving card {card_id:?}"));
    }

    fn move_card_with_result(&mut self, card_id: CardIdentifier) -> Result<GameResponseOutput> {
        self.perform_action(
            Action::MoveCard(MoveCardAction { card_id: Some(card_id), index: None }),
            self.user_id(),
        )
    }

    fn move_card_to_index(&mut self, card_id: CardIdentifier, index: u32) {
        self.move_card_to_index_with_result(card_id, index)
            .unwrap_or_else(|_| panic!("Error moving card {card_id:?} to index {index:?}"));
    }

    fn move_card_to_index_with_result(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
    ) -> Result<GameResponseOutput> {
        self.perform_action(
            Action::MoveCard(MoveCardAction { card_id: Some(card_id), index: Some(index) }),
            self.user_id(),
        )
    }

    fn move_to_end_step(&mut self, side: Side) {
        self.spend_all_action_points(side);
        self.click_as_side(Button::EndTurn, side);
    }

    fn pass_turn(&mut self, side: Side) {
        self.move_to_end_step(side);

        if self.side_has(Button::StartTurn, side.opponent()) {
            self.click_as_side(Button::StartTurn, side.opponent());
        }
    }

    fn dawn(&self) -> bool {
        assert_eq!(self.client.data.last_message(), self.opponent.data.last_message());
        self.client.data.last_message() == GameMessageType::Dawn
    }

    fn dusk(&self) -> bool {
        assert_eq!(self.client.data.last_message(), self.opponent.data.last_message());
        self.client.data.last_message() == GameMessageType::Dusk
    }

    fn is_victory_for_player(&self, winner: Side) -> bool {
        self.player_for_side(winner).data.last_message() == GameMessageType::Victory
            && self.player_for_side(winner.opponent()).data.last_message()
                == GameMessageType::Defeat
    }

    fn set_up_minion_combat(&mut self) {
        self.set_up_minion_combat_with_action(|_| {});
    }

    fn set_up_minion_combat_with_action(&mut self, action: impl FnOnce(&mut TestSession)) {
        self.create_and_play(CardName::TestScheme3_10);
        self.pass_turn(Side::Covenant);
        assert!(self.dawn());
        action(self);
        self.initiate_raid(test_constants::ROOM_ID);
        self.click_as_side(Button::Summon, Side::Covenant);
    }

    fn setup_raid_target(&mut self, card_name: CardName) -> (CardIdentifier, CardIdentifier) {
        self.pass_turn(Side::Riftcaller);
        assert!(self.dusk());
        let scheme_id = self.create_and_play(CardName::TestScheme3_10);
        let minion_id = self.create_and_play(card_name);
        self.pass_turn(Side::Covenant);
        assert!(self.dawn());
        (scheme_id, minion_id)
    }

    fn fire_weapon_combat_abilities(&mut self, resonance: Resonance, name: CardName) {
        self.setup_raid_target(crate::test_helpers::minion_for_resonance(resonance));
        self.initiate_raid(test_constants::ROOM_ID);
        self.click_as_side(Button::Summon, Side::Covenant);
        self.click_on(self.player_id_for_side(Side::Riftcaller), name.displayed_name());
    }
}

fn play_impl(
    session: &mut TestSession,
    card_variant: CardVariant,
    room_id: Option<RoomId>,
) -> CardIdentifier {
    let card_id = session.add_variant_to_hand(card_variant);
    session.play_card(
        card_id,
        session.player_id_for_side(test_game_client::side_for_card_name(card_variant.name)),
        room_id,
    );
    card_id
}

fn activate_ability_impl(
    session: &mut TestSession,
    card_id: CardIdentifier,
    index: u32,
    target: Option<RoomId>,
    player_id: PlayerId,
) -> Result<GameResponseOutput> {
    session.perform_action(
        Action::PlayCard(PlayCardAction {
            card_id: Some(CardIdentifier { ability_id: Some(index), ..card_id }),
            target: target.map(|room_id| CardTarget {
                card_target: Some(card_target::CardTarget::RoomId(adapters::room_identifier(
                    room_id,
                ))),
            }),
        }),
        player_id,
    )
}
