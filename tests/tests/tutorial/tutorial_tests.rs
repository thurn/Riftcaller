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
use protos::spelldawn::client_action::Action;
use protos::spelldawn::{
    ClientRoomLocation, DrawCardAction, GainManaAction, InitiateRaidAction, PlayerName,
    RoomIdentifier,
};
use test_utils::client::TestSession;
use test_utils::*;
use user_action_data::{NamedDeck, NewGameAction, NewGameDeck, UserAction};
static OPPONENT: PlayerId = PlayerId::Named(NamedPlayer::TutorialOpponent);

#[test]
fn set_up_tutorial() {
    let (game_id, user_id, _) = generate_ids();
    let mut session = new_session(game_id, user_id, OPPONENT);
    start_tutorial(&mut session);

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

#[test]
fn tutorial_turn_one() {
    let (game_id, user_id, _) = generate_ids();
    let mut session = new_session(game_id, user_id, OPPONENT);
    run_tutorial_turn_one(&mut session, user_id);
}

fn run_tutorial_turn_one(session: &mut TestSession, user_id: PlayerId) {
    start_tutorial(session);

    play_from_hand(session, CardName::EldritchSurge);
    assert!(session.user.data.toast().contains("Playing cards from your hand costs one"));

    play_from_hand(session, CardName::SimpleAxe);
    assert!(session.user.data.toast().contains("of the enemy's dungeon"));

    session.initiate_raid(RoomId::RoomA);

    // Opponent action: summon minion
    session.run_agent_loop();
    session.connect(session.user_id()).expect("Error reconnecting to session");

    let toast = session.user.data.toast();
    eprintln!("Toast is {toast:?}");
    assert!(session.user.data.toast().contains("To get past a defending minion"));

    session.click_on(user_id, CardName::SimpleAxe.displayed_name());
    assert!(session.user.data.toast().contains("You have accessed the room"));

    click_on_score(session);
    click_on_end_raid(session);

    session.run_agent_loop();
    session.connect(session.user_id()).expect("Error reconnecting to session");

    assert_eq!(cards(vec![CardName::ArcaneRecovery]), session.user.cards.hand(PlayerName::User));
    assert!(session.me().can_take_action());
}

#[test]
fn tutorial_cannot_raid_vault() {
    let (game_id, user_id, _) = generate_ids();
    let mut session = new_session(game_id, user_id, OPPONENT);
    start_tutorial(&mut session);
    let result = session.perform_action(
        Action::InitiateRaid(InitiateRaidAction { room_id: RoomIdentifier::Vault.into() }),
        session.user_id(),
    );

    assert!(result.is_err())
}

#[test]
fn tutorial_cannot_raid_sanctum() {
    let (game_id, user_id, _) = generate_ids();
    let mut session = new_session(game_id, user_id, OPPONENT);
    start_tutorial(&mut session);
    let result = session.perform_action(
        Action::InitiateRaid(InitiateRaidAction { room_id: RoomIdentifier::Sanctum.into() }),
        session.user_id(),
    );

    assert!(result.is_err())
}

#[test]
fn tutorial_turn_two() {
    let (game_id, user_id, _) = generate_ids();
    let mut session = new_session(game_id, user_id, OPPONENT);
    run_tutorial_turn_one(&mut session, user_id);
    run_tutorial_turn_two(&mut session, user_id);
}

fn run_tutorial_turn_two(session: &mut TestSession, user_id: PlayerId) {
    session.perform(Action::GainMana(GainManaAction {}), user_id);
    play_from_hand(session, CardName::ArcaneRecovery);
    session.perform(Action::DrawCard(DrawCardAction {}), user_id);
    session.run_agent_loop();
    session.connect(session.user_id()).expect("Error reconnecting to session");

    assert_eq!(
        cards(vec![CardName::Lodestone, CardName::SimpleHammer]),
        session.user.cards.hand(PlayerName::User)
    );
    assert!(session.me().can_take_action());
}

#[test]
fn tutorial_turn_three() {
    let (game_id, user_id, _) = generate_ids();
    let mut session = new_session(game_id, user_id, OPPONENT);
    run_tutorial_turn_one(&mut session, user_id);
    run_tutorial_turn_two(&mut session, user_id);
    run_tutorial_turn_three(&mut session, user_id);
}

fn run_tutorial_turn_three(session: &mut TestSession, user_id: PlayerId) {
    session.initiate_raid(RoomId::RoomA);

    // Opponent action: summon minion
    session.run_agent_loop();
    session.connect(session.user_id()).expect("Error reconnecting to session");

    click_on_continue(session);
    session.initiate_raid(RoomId::Vault);
    click_on_score(session);
    click_on_end_raid(session);
    session.perform(Action::DrawCard(DrawCardAction {}), user_id);

    session.run_agent_loop();
    session.connect(session.user_id()).expect("Error reconnecting to session");

    assert_eq!(
        cards(vec![
            CardName::Contemplate,
            CardName::Lodestone,
            CardName::SimpleClub,
            CardName::SimpleHammer
        ]),
        session.user.cards.hand(PlayerName::User)
    );
}

/// Initiates the tutorial.
///
/// *NOTE*: Opponent session state is not updated for the tutorial (it is
/// assumed to be single-player), and thus calls to `session.opponent` will not
/// have accurate information.
fn start_tutorial(session: &mut TestSession) {
    session
        .perform_action(
            UserAction::NewGame(NewGameAction {
                deck: NewGameDeck::NamedDeck(NamedDeck::TutorialChampion),
                opponent: OPPONENT,
                tutorial: true,
                debug_options: None,
            })
            .as_client_action(),
            session.user_id(),
        )
        .expect("Error starting tutorial");
    session.run_agent_loop();
    session.connect(session.user_id()).expect("Error reconnecting to session");
}

fn play_from_hand(session: &mut TestSession, card: CardName) {
    session.play_card(session.user.cards.find_in_user_hand(card), session.user_id(), None);
}

fn cards(vec: Vec<CardName>) -> Vec<String> {
    vec.iter().map(|c| c.displayed_name()).collect()
}
