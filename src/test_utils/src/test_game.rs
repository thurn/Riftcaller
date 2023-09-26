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

use std::iter;

use game_data::card_name::{CardName, CardVariant};
use game_data::card_state::{CardPosition, CardPositionKind};
use game_data::deck::Deck;
use game_data::game_state::{GameConfiguration, GamePhase, GameState, TurnData};
use game_data::game_updates::InitiatedBy;
use game_data::player_name::PlayerId;
use game_data::primitives::{ActionCount, GameId, ManaValue, PointsValue, Side};
use game_data::raid_data::{RaidData, RaidState, RaidStep};
use maplit::hashmap;
use rules::{dispatch, mana};

use crate::test_game_client;
use crate::test_session::TestSession;
use crate::test_session_builder::TestSessionBuilder;

pub struct TestGame {
    current_turn: Side,
    actions: ActionCount,
    raid: Option<TestRaid>,
    user_side: TestSide,
    opponent_side: TestSide,
    tutorial_mode: bool,
    deck_sizes: u32,
}

impl TestGame {
    /// Creates a new test game. Mulligans will be skipped and the game will be
    /// advanced directly to the `user_side` player's first turn without
    /// intervening events (e.g. the Overlord will not draw a card for their
    /// first turn if you pass [Side::Champion]).
    pub fn new(user_side: TestSide) -> Self {
        cards_all::initialize();
        let opponent = user_side.side.opponent();
        Self {
            current_turn: user_side.side,
            actions: 3,
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

    /// Action points for the player whose turn it is. Defaults to 3.
    pub fn actions(mut self, actions: ActionCount) -> Self {
        self.actions = actions;
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
        let (overlord_user, champion_user) = match self.user_side.side {
            Side::Overlord => (user_id, opponent_id),
            Side::Champion => (opponent_id, user_id),
        };

        let (overlord_riftcallers, champion_riftcallers) = match self.user_side.side {
            Side::Overlord => {
                (self.user_side.riftcallers.clone(), self.opponent_side.riftcallers.clone())
            }
            Side::Champion => {
                (self.opponent_side.riftcallers.clone(), self.user_side.riftcallers.clone())
            }
        };

        let overlord_deck = Deck {
            side: Side::Overlord,
            schools: vec![],
            riftcallers: overlord_riftcallers.into_iter().map(CardVariant::standard).collect(),
            cards: hashmap! {CardVariant::standard(CardName::TestOverlordSpell) => self.deck_sizes},
        };
        let champion_deck = Deck {
            side: Side::Champion,
            schools: vec![],
            riftcallers: champion_riftcallers.into_iter().map(CardVariant::standard).collect(),
            cards: hashmap! {CardVariant::standard(CardName::TestChampionSpell) => self.deck_sizes},
        };

        let mut game = GameState::new(
            game_id,
            overlord_user,
            overlord_deck,
            champion_user,
            champion_deck,
            GameConfiguration {
                deterministic: true,
                scripted_tutorial: self.tutorial_mode,
                ..GameConfiguration::default()
            },
        );

        dispatch::populate_delegate_cache(&mut game);

        game.info.phase = GamePhase::Play;
        game.info.turn = TurnData { side: self.current_turn, turn_number: 0 };

        self.user_side.apply_to(&mut game);
        self.opponent_side.apply_to(&mut game);
        game.player_mut(self.current_turn).actions = self.actions;

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
            accessed: vec![],
            jump_request: None,
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
    score: PointsValue,
    hand_size: usize,
    deck_top: Vec<CardName>,
    in_discard_face_down: Vec<CardName>,
    in_discard_face_up: Vec<CardName>,
    riftcallers: Vec<CardName>,
}

impl TestSide {
    pub fn new(side: Side) -> Self {
        TestSide {
            side,
            mana: test_constants::STARTING_MANA,
            score: 0,
            hand_size: 0,
            deck_top: vec![],
            in_discard_face_down: vec![],
            in_discard_face_up: vec![],
            riftcallers: vec![],
        }
    }

    /// Mana available for this player. Defaults to 999.
    pub fn mana(mut self, mana: ManaValue) -> Self {
        self.mana = mana;
        self
    }

    /// Score for this player. Defaults to 0.
    pub fn score(mut self, score: PointsValue) -> Self {
        self.score = score;
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

    /// Riftcallers which start in play for this player.
    pub fn riftcaller(mut self, card: CardName) -> Self {
        self.riftcallers.push(card);
        self
    }

    /// Starting size for this player's hand, draw from the top of
    /// their deck. Hand will consist entirely of 'test spell' cards.
    /// Defaults to 0.
    pub fn hand_size(mut self, hand_size: usize) -> Self {
        self.hand_size = hand_size;
        self
    }

    pub fn apply_to(&self, game: &mut GameState) {
        mana::set(game, self.side, self.mana);
        game.player_mut(self.side).score = self.score;

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

        let hand_card = if self.side == Side::Overlord {
            CardName::TestOverlordSpell
        } else {
            CardName::TestChampionSpell
        };
        let hand = iter::repeat(hand_card).take(self.hand_size).collect::<Vec<_>>();
        overwrite_positions(game, self.side, &hand, CardPosition::Hand(self.side), false);
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
