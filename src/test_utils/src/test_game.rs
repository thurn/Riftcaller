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

use std::iter;
use std::sync::atomic::Ordering;

use constants::game_constants;
use core_data::game_primitives::{
    ActionCount, CardPlayId, CurseCount, GameId, InitiatedBy, ManaValue, PointsValue, RoomId,
    RoomLocation, Side, WoundCount,
};
use dispatcher::dispatch;
use game_data::card_name::{CardName, CardVariant};
use game_data::card_state::{CardPosition, CardPositionKind};
use game_data::deck::Deck;
use game_data::game_state::{GameConfiguration, GamePhase, GameState, TurnData};
use game_data::player_name::PlayerId;
use game_data::raid_data::{RaidData, RaidState, RaidStep};
use game_data::utils;
use maplit::hashmap;

use crate::test_game_client;
use crate::test_session::TestSession;
use crate::test_session_builder::TestSessionBuilder;

pub struct TestGame {
    current_turn: Side,
    actions: Option<ActionCount>,
    raid: Option<TestRaid>,
    user_side: TestSide,
    opponent_side: TestSide,
    tutorial_mode: bool,
    deck_sizes: u32,
}

impl TestGame {
    /// Creates a new test game. Mulligans will be skipped and the game will be
    /// advanced directly to the `user_side` player's first turn without
    /// intervening events (e.g. the Covenant will not draw a card for their
    /// first turn if you pass [Side::Riftcaller]).
    pub fn new(user_side: TestSide) -> Self {
        cards_all::initialize();
        let opponent = user_side.side.opponent();
        Self {
            current_turn: user_side.side,
            actions: None,
            raid: None,
            user_side,
            opponent_side: TestSide::new(opponent),
            tutorial_mode: false,
            deck_sizes: 45,
        }
    }

    pub fn user_side(&self) -> Side {
        self.user_side.side
    }

    /// Player whose turn it should be. Defaults to the `user_side` player.
    pub fn current_turn(mut self, side: Side) -> Self {
        self.current_turn = side;
        self
    }

    /// Action points available for the player whose turn it is.
    pub fn actions(mut self, actions: ActionCount) -> Self {
        self.actions = Some(actions);
        self
    }

    pub fn raid(mut self, raid: TestRaid) -> Self {
        self.raid = Some(raid);
        self
    }

    pub fn opponent(mut self, side: TestSide) -> Self {
        self.opponent_side = side;
        self
    }

    pub fn tutorial_mode(mut self, tutorial_mode: bool) -> Self {
        self.tutorial_mode = tutorial_mode;
        self
    }

    pub fn deck_sizes(mut self, deck_sizes: u32) -> Self {
        self.deck_sizes = deck_sizes;
        self
    }

    /// Creates a new game with the user playing as the `user_side` player.
    ///
    /// By default, this creates a new game with both player's decks populated
    /// with blank test cards and all other game zones empty (no cards are
    /// drawn). The game is advanced to the user's first turn. See the other
    /// methods on this struct for information about the default configuration
    /// options and how to modify them.
    pub fn build(self) -> TestSession {
        TestSessionBuilder::new().game(self).build()
    }

    pub fn build_game_state_internal(
        self,
        game_id: GameId,
        user_id: PlayerId,
        opponent_id: PlayerId,
    ) -> GameState {
        let (covenant_user, riftcaller_user) = match self.user_side.side {
            Side::Covenant => (user_id, opponent_id),
            Side::Riftcaller => (opponent_id, user_id),
        };

        let (chapters, riftcallers) = match self.user_side.side {
            Side::Covenant => {
                (self.user_side.identities.clone(), self.opponent_side.identities.clone())
            }
            Side::Riftcaller => {
                (self.opponent_side.identities.clone(), self.user_side.identities.clone())
            }
        };

        let covenant_deck = Deck {
            side: Side::Covenant,
            schools: vec![],
            identities: chapters.into_iter().map(CardVariant::standard).collect(),
            sigils: vec![],
            cards: hashmap! {CardVariant::standard(CardName::TestRitual) => self.deck_sizes},
        };
        let riftcaller_deck = Deck {
            side: Side::Riftcaller,
            schools: vec![],
            identities: riftcallers.into_iter().map(CardVariant::standard).collect(),
            sigils: vec![],
            cards: hashmap! {CardVariant::standard(CardName::TestSpell) => self.deck_sizes},
        };

        let mut game = GameState::new(
            game_id,
            covenant_user,
            covenant_deck,
            riftcaller_user,
            riftcaller_deck,
            GameConfiguration {
                deterministic: true,
                scripted_tutorial: self.tutorial_mode,
                ..GameConfiguration::default()
            },
        );

        dispatch::populate_delegate_map(&mut game);

        game.info.phase = GamePhase::Play;
        game.info.turn = TurnData { side: self.current_turn, turn_number: 0 };

        self.user_side.apply_to(&mut game);
        self.opponent_side.apply_to(&mut game);
        game.player_mut(self.current_turn).actions =
            self.actions.unwrap_or(if self.user_side.side == Side::Covenant {
                game_constants::COVENANT_START_OF_TURN_ACTIONS
            } else {
                game_constants::RIFTCALLER_START_OF_TURN_ACTIONS
            });

        if let Some(r) = self.raid {
            r.apply_to(&mut game);
        }

        game
    }
}

pub struct TestRaid {}

impl TestRaid {
    pub fn new() -> Self {
        Self {}
    }

    pub fn apply_to(self, game: &mut GameState) {
        game.raid = Some(RaidData {
            raid_id: test_constants::RAID_ID,
            initiated_by: InitiatedBy::GameAction,
            target: test_constants::ROOM_ID,
            state: RaidState::Step(RaidStep::Begin),
            encounter: game.defenders_unordered(test_constants::ROOM_ID).count(),
            minion_encounter_id: None,
            room_access_id: None,
            accessed: vec![],
            jump_request: None,
            is_card_access_prevented: false,
            is_custom_access: false,
        })
    }
}

impl Default for TestRaid {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TestSide {
    side: Side,
    mana: ManaValue,
    bonus_points: PointsValue,
    hand_size: usize,
    in_hand: Vec<CardName>,
    curses: CurseCount,
    wounds: WoundCount,
    deck_top: Vec<CardName>,
    in_discard_face_down: Vec<CardName>,
    in_discard_face_up: Vec<CardName>,
    identities: Vec<CardName>,
    room_occupants: Vec<(RoomId, CardName)>,
    face_up_room_occupants: Vec<(RoomId, CardName)>,
    in_score_area: Vec<CardName>,
    face_up_defenders: Vec<(RoomId, CardName)>,
    face_down_defenders: Vec<(RoomId, CardName)>,
}

impl TestSide {
    pub fn new(side: Side) -> Self {
        TestSide {
            side,
            mana: test_constants::STARTING_MANA,
            bonus_points: 0,
            curses: 0,
            wounds: 0,
            hand_size: 0,
            in_hand: vec![],
            deck_top: vec![],
            in_discard_face_down: vec![],
            in_discard_face_up: vec![],
            identities: vec![],
            room_occupants: vec![],
            face_up_room_occupants: vec![],
            in_score_area: vec![],
            face_up_defenders: vec![],
            face_down_defenders: vec![],
        }
    }

    /// Mana available for this player. Defaults to 999.
    pub fn mana(mut self, mana: ManaValue) -> Self {
        self.mana = mana;
        self
    }

    /// Bonus points score for this player. Defaults to 0.
    pub fn bonus_points(mut self, score: PointsValue) -> Self {
        self.bonus_points = score;
        self
    }

    /// Card to be inserted into the player's hand
    ///
    /// If both this and [Self::hand_size] are specified, these cards are added
    /// in *addition* to the count provided for the `hand_size`.
    pub fn in_hand(mut self, card: CardName) -> Self {
        self.in_hand.push(card);
        self
    }

    /// Card to be inserted into the player's deck as the next draw.
    ///
    /// This card will be drawn when drawing randomly from the deck (as long as
    /// no known cards are placed on top of it) because the game is created as
    /// deterministic.
    pub fn deck_top(mut self, card: CardName) -> Self {
        self.deck_top.push(card);
        self
    }

    /// Card to be inserted face-down into the player's discard pile.
    pub fn in_discard_face_down(mut self, card: CardName) -> Self {
        self.in_discard_face_down.push(card);
        self
    }

    /// Card to be inserted face-up into the player's discard pile.
    pub fn in_discard_face_up(mut self, card: CardName) -> Self {
        self.in_discard_face_up.push(card);
        self
    }

    /// Covenant card to be inserted face-up into the player's score area.
    pub fn in_score_area(mut self, card: CardName) -> Self {
        self.in_score_area.push(card);
        self
    }

    /// Card to be inserted as a face-up defender of a room
    pub fn face_up_defender(mut self, room_id: RoomId, card: CardName) -> Self {
        self.face_up_defenders.push((room_id, card));
        self
    }

    /// Card to be inserted as a face-down defender of a room
    pub fn face_down_defender(mut self, room_id: RoomId, card: CardName) -> Self {
        self.face_down_defenders.push((room_id, card));
        self
    }

    /// Card to be inserted as a face-down occupant of a room
    pub fn room_occupant(mut self, room_id: RoomId, card: CardName) -> Self {
        self.room_occupants.push((room_id, card));
        self
    }

    /// Card to be inserted as a face-up occupant of a room
    pub fn face_up_room_occupant(mut self, room_id: RoomId, card: CardName) -> Self {
        self.face_up_room_occupants.push((room_id, card));
        self
    }

    /// Identity cards which start in play for this player.
    pub fn identity(mut self, card: CardName) -> Self {
        self.identities.push(card);
        self
    }

    /// Starting size for this player's hand, draw from the top of
    /// their deck. Hand will consist entirely of 'test spell' cards.
    /// Defaults to 0.
    pub fn hand_size(mut self, hand_size: usize) -> Self {
        self.hand_size = hand_size;
        self
    }

    pub fn curses(mut self, curses: CurseCount) -> Self {
        self.curses = curses;
        self
    }

    pub fn wounds(mut self, wounds: WoundCount) -> Self {
        self.wounds = wounds;
        self
    }

    pub fn apply_to(&self, game: &mut GameState) {
        game.player_mut(self.side).mana_state.base_mana = self.mana;
        game.player_mut(self.side).bonus_points = self.bonus_points;
        game.player_mut(self.side).curse_state.base_curses = self.curses;
        game.player_mut(self.side).wounds = self.wounds;

        overwrite_positions(
            game,
            self.side,
            &self.deck_top,
            CardPosition::DeckTop(self.side),
            false,
        );
        overwrite_positions(
            game,
            self.side,
            &self.in_discard_face_down,
            CardPosition::DiscardPile(self.side),
            false,
        );
        overwrite_positions(
            game,
            self.side,
            &self.in_discard_face_up,
            CardPosition::DiscardPile(self.side),
            true,
        );
        for (room_id, card_name) in &self.room_occupants {
            overwrite_positions(
                game,
                Side::Covenant,
                &[*card_name],
                CardPosition::Room(card_play_id(), *room_id, RoomLocation::Occupant),
                false,
            );
        }
        for (room_id, card_name) in &self.face_up_room_occupants {
            overwrite_positions(
                game,
                Side::Covenant,
                &[*card_name],
                CardPosition::Room(card_play_id(), *room_id, RoomLocation::Occupant),
                true,
            );
        }
        overwrite_positions(
            game,
            Side::Covenant,
            &self.in_score_area,
            CardPosition::Scored(self.side),
            true,
        );
        for (room_id, card_name) in &self.face_up_defenders {
            overwrite_positions(
                game,
                Side::Covenant,
                &[*card_name],
                CardPosition::Room(card_play_id(), *room_id, RoomLocation::Defender),
                true,
            );
        }
        for (room_id, card_name) in &self.face_down_defenders {
            overwrite_positions(
                game,
                Side::Covenant,
                &[*card_name],
                CardPosition::Room(card_play_id(), *room_id, RoomLocation::Defender),
                false,
            );
        }

        let hand_card =
            if self.side == Side::Covenant { CardName::TestRitual } else { CardName::TestSpell };
        let hand = iter::repeat(hand_card).take(self.hand_size).collect::<Vec<_>>();
        overwrite_positions(game, self.side, &hand, CardPosition::Hand(self.side), false);
        overwrite_positions(game, self.side, &self.in_hand, CardPosition::Hand(self.side), false);
    }
}

fn overwrite_positions(
    game: &mut GameState,
    side: Side,
    cards: &[CardName],
    position: CardPosition,
    turn_face_up: bool,
) {
    for card in cards {
        let target_id = game
            .cards(side)
            .iter()
            .find(|c| c.position().kind() == CardPositionKind::DeckUnknown)
            .expect("No cards in deck")
            .id;
        test_game_client::overwrite_card(game, target_id, CardVariant::standard(*card));
        game.move_card_internal(target_id, position);

        if turn_face_up {
            game.card_mut(target_id).internal_turn_face_up();
        } else {
            game.card_mut(target_id).internal_turn_face_down();
        }
    }
}

fn card_play_id() -> CardPlayId {
    CardPlayId(utils::DEBUG_EVENT_ID.fetch_add(1, Ordering::Relaxed))
}
