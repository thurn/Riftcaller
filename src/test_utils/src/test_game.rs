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

use game_data::card_name::CardName;
use game_data::primitives::{ActionCount, ManaValue, PointsValue, Side};

use crate::test_session::TestSession;
use crate::{AdventureArgs, Args, STARTING_MANA};

pub struct TestGame {
    current_turn: Side,
    actions: ActionCount,
    raid: Option<TestRaid>,
    user_side: TestSide,
    opponent_side: TestSide,
    tutorial_mode: bool,
    connect: bool,
    adventure: Option<AdventureArgs>,
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
            connect: true,
            adventure: None,
        }
    }

    /// Player whose turn it should be. Defaults to the `user_side` player.
    pub fn current_turn(mut self, side: Side) -> Self {
        self.current_turn = side;
        self
    }

    /// Actions points for the player whose turn it is. Defaults to 3.
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

    pub fn connect(mut self, connect: bool) -> Self {
        self.connect = connect;
        self
    }

    pub fn adventure(mut self, adventure: AdventureArgs) -> Self {
        self.adventure = Some(adventure);
        self
    }

    #[allow(clippy::all)]
    pub fn build(&self) -> TestSession {
        let mut args = Args::default();
        args.turn = Some(self.current_turn);
        args.mana = self.user_side.mana;
        args.opponent_mana = self.opponent_side.mana;
        args.actions = self.actions;
        args.score = self.user_side.score;
        args.opponent_score = self.opponent_side.score;
        args.hand_size = self.user_side.hand_size;
        args.opponent_hand_size = self.opponent_side.hand_size;
        args.deck_top = self.user_side.deck_top.clone();
        args.opponent_deck_top = self.opponent_side.deck_top.clone();
        args.discard = self.user_side.get_discard();
        args.opponent_discard = self.opponent_side.get_discard();
        args.sigils = self.user_side.sigils.clone();
        args.opponent_sigils = self.opponent_side.sigils.clone();
        args.add_raid = self.raid.is_some();
        args.tutorial = self.tutorial_mode;
        args.connect = self.connect;
        args.adventure = self.adventure.clone();

        crate::new_game(self.user_side.side, args)
    }
}

pub struct TestRaid {}

impl TestRaid {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TestRaid {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TestSide {
    pub side: Side,
    pub mana: ManaValue,
    pub score: PointsValue,
    pub hand_size: u64,
    pub deck_top: Vec<CardName>,
    pub discard: Vec<CardName>,
    pub sigils: Vec<CardName>,
}

impl TestSide {
    pub fn new(side: Side) -> Self {
        TestSide {
            side,
            mana: STARTING_MANA,
            score: 0,
            hand_size: 0,
            deck_top: vec![],
            discard: vec![],
            sigils: vec![],
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

    /// Card to be inserted into the player's discard pile.
    pub fn discard(mut self, card: CardName) -> Self {
        self.discard.push(card);
        self
    }

    pub fn get_discard(&self) -> Option<CardName> {
        if self.discard.is_empty() {
            None
        } else {
            Some(self.discard[0])
        }
    }

    /// Sigils which start in play for this player.
    pub fn sigil(mut self, card: CardName) -> Self {
        self.sigils.push(card);
        self
    }

    /// Starting size for this player's hand, draw from the top of
    /// their deck. Hand will consist entirely of 'test spell' cards.
    /// Defaults to 0.
    pub fn hand_size(mut self, hand_size: u64) -> Self {
        self.hand_size = hand_size;
        self
    }
}
