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

use core_data::game_primitives::Side;
use core_ui::actions::InterfaceAction;
use game_data::game_actions::{GameAction, GameStateAction};
use game_data::game_state::MulliganDecision;
use insta::assert_snapshot;
use test_utils::client_interface::HasText;
use test_utils::summarize::Summary;
use test_utils::test_session::TestSession;
use test_utils::test_session_builder::TestSessionBuilder;
use test_utils::*;
use user_action_data::{NamedDeck, NewGameAction, NewGameDebugOptions, NewGameDeck, UserAction};

static COVENANT_DECK: NewGameDeck = NewGameDeck::NamedDeck(NamedDeck::CovenantTestSpells);
static RIFTCALLER_DECK: NewGameDeck = NewGameDeck::NamedDeck(NamedDeck::RiftcallerTestSpells);

#[test]
fn create_new_game() {
    let (game_id, covenant_id, riftcaller_id) = test_helpers::generate_ids();
    let mut session = TestSessionBuilder::new()
        .game_id(game_id)
        .user_id(covenant_id)
        .opponent_id(riftcaller_id)
        .do_not_connect(true)
        .build();
    let response = session.perform_action(
        UserAction::NewGame(NewGameAction {
            deck: COVENANT_DECK,
            opponent: session.opponent_id(),
            debug_options: Some(NewGameDebugOptions {
                deterministic: true,
                ..NewGameDebugOptions::default()
            }),
            tutorial: false,
        })
        .as_client_action(),
        session.user_id(),
    );
    assert_snapshot!(Summary::run(&response));
}

#[test]
fn connect_to_new_game() {
    let (game_id, covenant_id, riftcaller_id) = test_helpers::generate_ids();
    let mut session = TestSessionBuilder::new()
        .game_id(game_id)
        .user_id(covenant_id)
        .opponent_id(riftcaller_id)
        .do_not_connect(true)
        .build();
    initiate_game(&mut session);

    let response = session.connect(covenant_id);
    assert!(session.client.interface.controls().has_text("Keep"));
    assert!(session.client.interface.controls().has_text("Mulligan"));
    assert_eq!(5, session.client.cards.revealed_cards_browser().len());
    assert_eq!(5, session.client.cards.opponent_hand().len());

    assert_snapshot!(Summary::run(&response));
}

#[test]
fn mulligan_legal_actions() {
    let (game_id, covenant_id, riftcaller_id) = test_helpers::generate_ids();
    let mut session = TestSessionBuilder::new()
        .game_id(game_id)
        .user_id(covenant_id)
        .opponent_id(riftcaller_id)
        .do_not_connect(true)
        .build();
    initiate_game(&mut session);

    test_helpers::assert_contents_equal(
        session.legal_actions(Side::Covenant),
        vec![
            GameAction::GameStateAction(GameStateAction::MulliganDecision(
                MulliganDecision::Mulligan,
            )),
            GameAction::GameStateAction(GameStateAction::MulliganDecision(MulliganDecision::Keep)),
        ],
    );
    test_helpers::assert_contents_equal(
        session.legal_actions(Side::Riftcaller),
        vec![
            GameAction::GameStateAction(GameStateAction::MulliganDecision(
                MulliganDecision::Mulligan,
            )),
            GameAction::GameStateAction(GameStateAction::MulliganDecision(MulliganDecision::Keep)),
        ],
    );

    session.click_on(covenant_id, "Keep");
    assert!(session.legal_actions_result(Side::Covenant).is_err());
    test_helpers::assert_contents_equal(
        session.legal_actions(Side::Riftcaller),
        vec![
            GameAction::GameStateAction(GameStateAction::MulliganDecision(
                MulliganDecision::Mulligan,
            )),
            GameAction::GameStateAction(GameStateAction::MulliganDecision(MulliganDecision::Keep)),
        ],
    );
}

#[test]
fn keep_opening_hand() {
    let (game_id, covenant_id, riftcaller_id) = test_helpers::generate_ids();
    let mut session = TestSessionBuilder::new()
        .game_id(game_id)
        .user_id(covenant_id)
        .opponent_id(riftcaller_id)
        .do_not_connect(true)
        .build();
    initiate_game(&mut session);

    let response = session.click_on(covenant_id, "Keep");
    assert_eq!(0, session.client.cards.revealed_cards_browser().len());
    assert_eq!(5, session.client.cards.hand().len());
    assert_eq!(5, session.client.cards.opponent_hand().len());

    assert_eq!(0, session.opponent.cards.hand().len());
    assert_eq!(5, session.opponent.cards.opponent_hand().len());
    assert_eq!(5, session.opponent.cards.revealed_cards_browser().len());

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn mulligan_opening_hand() {
    let (game_id, covenant_id, riftcaller_id) = test_helpers::generate_ids();
    let mut session = TestSessionBuilder::new()
        .game_id(game_id)
        .user_id(covenant_id)
        .opponent_id(riftcaller_id)
        .do_not_connect(true)
        .build();
    initiate_game(&mut session);

    let response = session.click_on(covenant_id, "Mulligan");
    assert_snapshot!(Summary::summarize(&response));

    assert_eq!(0, session.client.cards.revealed_cards_browser().len());
    assert_eq!(5, session.client.cards.hand().len());
    assert_eq!(5, session.client.cards.opponent_hand().len());

    assert_eq!(0, session.opponent.cards.hand().len());
    assert_eq!(5, session.opponent.cards.opponent_hand().len());
    assert_eq!(5, session.opponent.cards.revealed_cards_browser().len());
}

#[test]
fn both_keep_opening_hands() {
    let (game_id, covenant_id, riftcaller_id) = test_helpers::generate_ids();
    let mut session = TestSessionBuilder::new()
        .game_id(game_id)
        .user_id(covenant_id)
        .opponent_id(riftcaller_id)
        .do_not_connect(true)
        .build();
    initiate_game(&mut session);

    session.click_on(covenant_id, "Keep");
    let response = session.click_on(riftcaller_id, "Keep");
    assert_snapshot!(Summary::summarize(&response));

    assert_eq!(5, session.client.this_player.mana());
    assert_eq!(5, session.client.other_player.mana());
    assert_eq!(5, session.opponent.this_player.mana());
    assert_eq!(5, session.opponent.other_player.mana());

    assert_eq!(3, session.client.this_player.actions());
    assert_eq!(0, session.client.other_player.actions());
    assert_eq!(0, session.opponent.this_player.actions());
    assert_eq!(3, session.opponent.other_player.actions());

    assert_eq!(6, session.client.cards.hand().len());
    assert_eq!(5, session.client.cards.opponent_hand().len());
    assert_eq!(5, session.opponent.cards.hand().len());
    assert_eq!(6, session.opponent.cards.opponent_hand().len());

    assert!(session.dusk());
}

fn initiate_game(session: &mut TestSession) {
    session.perform(
        UserAction::NewGame(NewGameAction {
            deck: RIFTCALLER_DECK,
            opponent: session.user_id(),
            debug_options: Some(NewGameDebugOptions {
                deterministic: true,
                ..NewGameDebugOptions::default()
            }),
            tutorial: false,
        })
        .as_client_action(),
        session.opponent_id(),
    );
    session.perform(
        UserAction::NewGame(NewGameAction {
            deck: COVENANT_DECK,
            opponent: session.opponent_id(),
            debug_options: Some(NewGameDebugOptions {
                deterministic: true,
                ..NewGameDebugOptions::default()
            }),
            tutorial: false,
        })
        .as_client_action(),
        session.user_id(),
    );

    session.connect(session.user_id()).unwrap();
    session.connect(session.opponent_id()).unwrap();
}
