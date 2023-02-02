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

use core_ui::actions::InterfaceAction;
use game_data::card_name::CardName;
use game_data::player_name::{NamedPlayer, PlayerId};
use game_data::primitives::RoomId;
use protos::spelldawn::{ClientRoomLocation, PlayerName};
use test_utils::client::TestSession;
use test_utils::*;
use user_action_data::{NamedDeck, NewGameAction, NewGameDeck, UserAction};
static OPPONENT: PlayerId = PlayerId::Named(NamedPlayer::TestAlphaBetaHeuristics);

#[tokio::test]
async fn set_up_tutorial() {
    let (game_id, user_id, _) = generate_ids();
    let mut session = new_session(game_id, user_id, OPPONENT);
    start_tutorial(&mut session).await;

    assert_eq!(
        cards(vec![CardName::EldritchSurge, CardName::SimpleAxe]),
        session.user.cards.hand(PlayerName::User)
    );
    assert_eq!(1, session.user.cards.hand(PlayerName::Opponent).len());
    assert!(session.me().can_take_action());
    assert!(session.user.data.toast().contains("lets you play cards and use weapons"));
    assert_eq!(1, session.user.cards.room_cards(RoomId::RoomA, ClientRoomLocation::Front).len());
    assert_eq!(1, session.user.cards.room_cards(RoomId::RoomA, ClientRoomLocation::Back).len());
}

#[tokio::test]
async fn tutorial_turn_one() {
    let (game_id, user_id, _) = generate_ids();
    let mut session = new_session(game_id, user_id, OPPONENT);
    start_tutorial(&mut session).await;

    session.play_card(
        session.user.cards.find_in_user_hand(CardName::EldritchSurge),
        session.user_id(),
        None,
    );
    assert!(session.user.data.toast().contains("Playing cards from your hand costs one"));

    session.play_card(
        session.user.cards.find_in_user_hand(CardName::SimpleAxe),
        session.user_id(),
        None,
    );
    assert!(session.user.data.toast().contains("of the enemy's dungeon"));

    session.initiate_raid(RoomId::RoomA);
    assert!(session.user.data.toast().contains("To get past a defending minion"));
}

/// Initiates the tutorial.
///
/// *NOTE*: Opponent session state is not updated for the tutorial (it is
/// assumed to be single-player), and thus calls to `session.opponent` will not
/// have accurate information.
async fn start_tutorial(session: &mut TestSession) {
    session
        .perform_action(
            UserAction::NewGame(NewGameAction {
                deck: NewGameDeck::NamedDeck(NamedDeck::BasicChampion),
                opponent: OPPONENT,
                tutorial: true,
                debug_options: None,
            })
            .as_client_action(),
            session.user_id(),
        )
        .expect("Error starting tutorial");
    session.run_agent_loop().await
}

fn cards(vec: Vec<CardName>) -> Vec<String> {
    vec.iter().map(|c| c.displayed_name()).collect()
}
