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

use game_data::card_name::CardName;
use game_data::player_name::PlayerId;
use game_data::primitives::{CardType, Lineage, RoomId, Side};
use protos::spelldawn::client_action::Action;
use protos::spelldawn::{
    card_target, CardIdentifier, CardTarget, GameMessageType, InitiateRaidAction,
    LevelUpRoomAction, PlayCardAction, SpendActionPointAction,
};
use server::server_data::GameResponseOutput;

use crate::test_game_client::{ClientPlayer, TestGameClient};
use crate::test_session::TestSession;
use crate::{test_game_client, Button, TestInterfaceHelpers};

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

    /// Helper function to perform an action to initiate a raid on the provided
    /// `room_id`.
    fn initiate_raid(&mut self, room_id: RoomId) -> GameResponseOutput;

    /// Helper function to perform an action to level up the
    /// provided `room_id`.
    fn level_up_room(&mut self, room_id: RoomId) -> GameResponseOutput;

    /// Levels up the [test_constants::CLIENT_ROOM_ID] room a specified number
    /// of `times`. If this requires multiple turns, spends the Champion turns
    /// doing nothing.
    fn level_up_room_times(&mut self, times: u32);

    /// Helper to take the [PlayCardAction] with a given card ID.
    fn play_card(
        &mut self,
        card_id: CardIdentifier,
        player_id: PlayerId,
        target: Option<CardTarget>,
    );

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

    /// Equivalent method to [Self::create_and_play] which specifies
    /// a target room to use.
    fn play_with_target_room(&mut self, card_name: CardName, room_id: RoomId) -> CardIdentifier;

    /// Activates an ability of a card owned by the user based on its ability
    /// index.
    fn activate_ability(&mut self, card_id: CardIdentifier, index: u32);

    /// Activates an ability of a card with a target room
    fn activate_ability_with_target(&mut self, card_id: CardIdentifier, index: u32, target: RoomId);

    /// Unveils a card in play, paying its mana cost and turning it face up.
    fn unveil_card(&mut self, card_id: CardIdentifier);

    /// Spends one of the `side` player's action points with no effect
    fn spend_action_point(&mut self, side: Side);

    /// Spends all of the `side` player's action points with no effect
    fn spend_all_action_points(&mut self, side: Side);

    /// Spends the `side` player's action points with no effect until they have
    /// no action points remaining and then clicks the "End Turn" button.
    fn to_end_step(&mut self, side: Side);

    /// Spends the `side` player's action points with no effect until they have
    /// no action points remaining, clicks on "End Turn", and then starts the
    /// next player's turn.
    fn pass_turn(&mut self, side: Side);

    /// Returns true if the last-received Game Message was 'Dawn'.
    fn dawn(&self) -> bool;

    /// Returns true if the last-received Game Message was 'Dusk'.
    fn dusk(&self) -> bool;

    /// Returns true if the last-received Game Messages indicated the `winner`
    /// player won the game
    fn is_victory_for_player(&self, winner: Side) -> bool;

    /// Must be invoked during the Overlord turn. Performs the following
    /// actions:
    /// - Plays a test Scheme card
    /// - Ends the Overlord turn
    /// - Initiates a raid on the [test_constants::ROOM_ID] room
    /// - Summons the minion in the room
    fn set_up_minion_combat(&mut self);

    /// Equivalent to [Self::set_up_minion_combat] which invokes an `action`
    /// function
    /// at the start of the Champion's turn.
    fn set_up_minion_combat_with_action(&mut self, action: impl FnOnce(&mut TestSession));

    /// Must be invoked during the Champion turn. Performs the following
    /// actions:
    ///
    /// - Ends the Champion turn
    /// - Plays a 3-1 scheme in the [test_constants::ROOM_ID] room.
    /// - Plays the provided `card_name` minion into that room.
    /// - Plays the selected minion in the [test_constants::ROOM_ID] room.
    /// - Ends the Overlord turn.
    ///
    /// Returns a tuple of (scheme_id, minion_id).
    ///
    /// WARNING: This causes the Overlord player to draw for their turn.
    fn setup_raid_target(&mut self, card_name: CardName) -> (CardIdentifier, CardIdentifier);

    /// Must be invoked during the Champion turn. Performs the following
    /// actions:
    ///
    /// - Performs all actions described in [Self::setup_raid_target], creating
    ///   a minion of the indicated [Lineage] with `MINION_HEALTH` health.
    /// - Initiates a raid on the [test_constants::ROOM_ID] room.
    /// - Summons the minion
    /// - Clicks on the button with text matching `name` in order to fire weapon
    ///   abilities.
    ///
    /// WARNING: This causes the Overlord play to draw for their turn.
    fn fire_weapon_combat_abilities(&mut self, lineage: Lineage, name: CardName);
}

impl TestSessionHelpers for TestSession {
    fn user_id(&self) -> PlayerId {
        self.user.id
    }

    fn opponent_id(&self) -> PlayerId {
        self.opponent.id
    }

    fn player(&self, player_id: PlayerId) -> &TestGameClient {
        match () {
            _ if player_id == self.user.id => &self.user,
            _ if player_id == self.opponent.id => &self.opponent,
            _ => panic!("Unknown player id: {player_id:?}"),
        }
    }

    fn player_for_side(&self, side: Side) -> &TestGameClient {
        self.player(self.player_id_for_side(side))
    }

    fn me(&self) -> &ClientPlayer {
        &self.user.this_player
    }

    fn you(&self) -> &ClientPlayer {
        &self.opponent.this_player
    }

    fn perform(&mut self, action: Action, user_id: PlayerId) {
        self.perform_action(action, user_id).expect("Request failed");
    }

    fn initiate_raid(&mut self, room_id: RoomId) -> GameResponseOutput {
        self.perform_action(
            Action::InitiateRaid(InitiateRaidAction {
                room_id: adapters::room_identifier(room_id),
            }),
            self.player_id_for_side(Side::Champion),
        )
        .expect("Server Error")
    }

    fn level_up_room(&mut self, room_id: RoomId) -> GameResponseOutput {
        self.perform_action(
            Action::LevelUpRoom(LevelUpRoomAction { room_id: adapters::room_identifier(room_id) }),
            self.player_id_for_side(Side::Overlord),
        )
        .expect("Server Error")
    }

    fn level_up_room_times(&mut self, times: u32) {
        let mut levels = 0;
        let overlord_id = self.player_id_for_side(Side::Overlord);

        loop {
            while self.player(overlord_id).this_player.actions() > 0 {
                self.perform(
                    Action::LevelUpRoom(LevelUpRoomAction {
                        room_id: test_constants::CLIENT_ROOM_ID.into(),
                    }),
                    overlord_id,
                );
                levels += 1;

                if levels == times {
                    return;
                }
            }

            self.pass_turn(Side::Overlord);
            assert!(self.dawn());
            self.pass_turn(Side::Champion);
            assert!(self.dusk());
        }
    }

    fn play_card(
        &mut self,
        card_id: CardIdentifier,
        player_id: PlayerId,
        target: Option<CardTarget>,
    ) {
        self.perform(
            Action::PlayCard(PlayCardAction { card_id: Some(card_id), target }),
            player_id,
        );
    }

    fn create_and_play(&mut self, card_name: CardName) -> CardIdentifier {
        play_impl(
            self,
            card_name,
            match rules::get(card_name).card_type {
                CardType::Minion | CardType::Project | CardType::Scheme => {
                    Some(test_constants::ROOM_ID)
                }
                _ => None,
            },
        )
    }

    fn play_with_target_room(&mut self, card_name: CardName, room_id: RoomId) -> CardIdentifier {
        play_impl(self, card_name, Some(room_id))
    }

    fn activate_ability(&mut self, card_id: CardIdentifier, index: u32) {
        activate_ability_impl(self, card_id, index, None)
    }

    fn activate_ability_with_target(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
        target: RoomId,
    ) {
        activate_ability_impl(self, card_id, index, Some(target))
    }

    fn unveil_card(&mut self, card_id: CardIdentifier) {
        let id = CardIdentifier { is_unveil: true, ..card_id };
        self.perform(
            Action::PlayCard(PlayCardAction { card_id: Some(id), target: None }),
            self.player_id_for_side(adapters::side(id.side).expect("Invalid Side")),
        );
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

    fn to_end_step(&mut self, side: Side) {
        let id = self.player_id_for_side(side);
        self.spend_all_action_points(side);
        self.click_on(id, "End Turn");
    }

    fn pass_turn(&mut self, side: Side) {
        self.to_end_step(side);
        let opponent_id = self.player_id_for_side(side.opponent());
        self.click_on(opponent_id, "Start Turn");
    }

    fn dawn(&self) -> bool {
        assert_eq!(self.user.data.last_message(), self.opponent.data.last_message());
        self.user.data.last_message() == GameMessageType::Dawn
    }

    fn dusk(&self) -> bool {
        assert_eq!(self.user.data.last_message(), self.opponent.data.last_message());
        self.user.data.last_message() == GameMessageType::Dusk
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
        self.create_and_play(CardName::TestScheme3_15);
        self.pass_turn(Side::Overlord);
        assert!(self.dawn());
        action(self);
        self.initiate_raid(test_constants::ROOM_ID);
        self.click_as_side(Button::Summon, Side::Overlord);
    }

    fn setup_raid_target(&mut self, card_name: CardName) -> (CardIdentifier, CardIdentifier) {
        self.pass_turn(Side::Champion);
        assert!(self.dusk());
        let scheme_id = self.create_and_play(CardName::TestScheme3_15);
        let minion_id = self.create_and_play(card_name);
        self.pass_turn(Side::Overlord);
        assert!(self.dawn());
        (scheme_id, minion_id)
    }

    fn fire_weapon_combat_abilities(&mut self, lineage: Lineage, name: CardName) {
        self.setup_raid_target(crate::test_helpers::minion_for_lineage(lineage));
        self.initiate_raid(test_constants::ROOM_ID);
        self.click_as_side(Button::Summon, Side::Overlord);
        self.click_on(self.player_id_for_side(Side::Champion), name.displayed_name());
    }
}

fn play_impl(
    session: &mut TestSession,
    card_name: CardName,
    room_id: Option<RoomId>,
) -> CardIdentifier {
    let card_id = session.add_to_hand(card_name);
    let target = room_id.map(|room_id| CardTarget {
        card_target: Some(card_target::CardTarget::RoomId(adapters::room_identifier(room_id))),
    });

    session.play_card(
        card_id,
        session.player_id_for_side(test_game_client::side_for_card_name(card_name)),
        target,
    );

    card_id
}

fn activate_ability_impl(
    session: &mut TestSession,
    card_id: CardIdentifier,
    index: u32,
    target: Option<RoomId>,
) {
    session.perform(
        Action::PlayCard(PlayCardAction {
            card_id: Some(CardIdentifier { ability_id: Some(index), ..card_id }),
            target: target.map(|room_id| CardTarget {
                card_target: Some(card_target::CardTarget::RoomId(adapters::room_identifier(
                    room_id,
                ))),
            }),
        }),
        session.user_id(),
    );
}
