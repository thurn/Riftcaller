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

//! Tools to facilitate testing. Should be included via wildcard import in all
//! tests.

#![allow(clippy::unwrap_in_result)]

pub mod client_interface;
pub mod fake_database;
pub mod summarize;
pub mod test_adventure;
pub mod test_game;
pub mod test_helpers;
pub mod test_session;
pub mod test_session_helpers;

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use adventure_data::adventure::{
    AdventureConfiguration, AdventureState, BattleData, Coins, TileEntity,
};
use adventure_generator::mock_adventure;
use game_data::card_name::CardName;
use game_data::card_state::{CardPosition, CardPositionKind};
use game_data::character_preset::{CharacterFacing, CharacterPreset};
use game_data::deck::Deck;
use game_data::game::{
    GameConfiguration, GamePhase, GameState, InternalRaidPhase, RaidData, TurnData,
};
use game_data::player_name::{AIPlayer, PlayerId};
use game_data::primitives::{
    ActionCount, AdventureId, GameId, ManaValue, PointsValue, RaidId, RoomId, Side,
};
use game_data::tutorial_data::TutorialData;
use maplit::hashmap;
use player_data::{PlayerState, PlayerStatus};
use protos::spelldawn::RoomIdentifier;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;
use rules::{dispatch, mana};
pub use test_session_helpers::{Buttons, TestSessionHelpers};

use crate::fake_database::FakeDatabase;
use crate::test_session::TestSession;

pub static NEXT_ID: AtomicU64 = AtomicU64::new(1_000_000);
/// The title returned for hidden cards
pub const HIDDEN_CARD: &str = "Hidden Card";
/// [RoomId] used by default for targeting
pub const ROOM_ID: RoomId = RoomId::RoomA;
/// Client equivalent of [ROOM_ID].
pub const CLIENT_ROOM_ID: RoomIdentifier = RoomIdentifier::RoomA;
/// Default Raid ID to use during testing
pub const RAID_ID: RaidId = RaidId(1);
/// Default mana for players in a test game if not otherwise specified
pub const STARTING_MANA: ManaValue = 999;

/// Creates a new game with the user playing as the `user_side` player.
///
/// By default, this creates a new game with both player's decks populated with
/// blank test cards and all other game zones empty (no cards are drawn). The
/// game is advanced to the user's first turn. See [Args] for information about
/// the default configuration options and how to modify them.
pub fn new_game(user_side: Side, args: Args) -> TestSession {
    cards_all::initialize();
    let (game_id, user_id, opponent_id) = test_helpers::generate_ids();
    let (overlord_user, champion_user) = match user_side {
        Side::Overlord => (user_id, opponent_id),
        Side::Champion => (opponent_id, user_id),
    };

    let (overlord_sigils, champion_sigils) = match user_side {
        Side::Overlord => (args.sigils, args.opponent_sigils),
        Side::Champion => (args.opponent_sigils, args.sigils),
    };

    let overlord_deck = Deck {
        side: Side::Overlord,
        schools: vec![],
        sigils: overlord_sigils,
        cards: hashmap! {CardName::TestOverlordSpell => 45},
    };
    let champion_deck = Deck {
        side: Side::Champion,
        schools: vec![],
        sigils: champion_sigils,
        cards: hashmap! {CardName::TestChampionSpell => 45},
    };

    let mut game = GameState::new(
        game_id,
        overlord_user,
        overlord_deck,
        champion_user,
        champion_deck,
        GameConfiguration {
            deterministic: true,
            scripted_tutorial: args.tutorial,
            ..GameConfiguration::default()
        },
    );
    dispatch::populate_delegate_cache(&mut game);

    let turn_side = args.turn.unwrap_or(user_side);
    game.info.phase = GamePhase::Play;
    game.info.turn = TurnData { side: turn_side, turn_number: 0 };
    mana::set(&mut game, user_side, args.mana);
    game.player_mut(user_side).score = args.score;
    mana::set(&mut game, user_side.opponent(), args.opponent_mana);
    game.player_mut(user_side.opponent()).score = args.opponent_score;
    game.player_mut(turn_side).actions = args.actions;

    set_deck_top(&mut game, user_side, args.deck_top);
    set_deck_top(&mut game, user_side.opponent(), args.opponent_deck_top);
    set_discard_pile(&mut game, user_side, args.discard);
    set_discard_pile(&mut game, user_side.opponent(), args.opponent_discard);

    if args.add_raid {
        game.info.raid = Some(RaidData {
            raid_id: RAID_ID,
            target: ROOM_ID,
            internal_phase: InternalRaidPhase::Begin,
            encounter: None,
            accessed: vec![],
            jump_request: None,
        })
    }

    let (overlord_adventure, champion_adventure) = if let Some(adventure_args) = args.adventure {
        let adventure = create_mock_adventure(user_id, user_side, adventure_args);
        match user_side {
            Side::Overlord => (Some(adventure), None),
            Side::Champion => (None, Some(adventure)),
        }
    } else {
        (None, None)
    };

    let database = FakeDatabase {
        generated_game_id: None,
        game: Mutex::new(Some(game)),
        players: Mutex::new(hashmap! {
            overlord_user => PlayerState {
                id: overlord_user,
                status: Some(PlayerStatus::Playing(game_id)),
                adventure: overlord_adventure,
                tutorial: TutorialData::default()
            },
            champion_user => PlayerState {
                id: champion_user,
                status: Some(PlayerStatus::Playing(game_id)),
                adventure: champion_adventure,
                tutorial: TutorialData::default()
            }
        }),
    };

    let mut session = TestSession::new(database, user_id, opponent_id);
    let (user_hand_card, opponent_hand_card) = if user_side == Side::Overlord {
        (CardName::TestOverlordSpell, CardName::TestChampionSpell)
    } else {
        (CardName::TestChampionSpell, CardName::TestOverlordSpell)
    };

    for _ in 0..args.hand_size {
        session.add_to_hand(user_hand_card);
    }
    for _ in 0..args.opponent_hand_size {
        session.add_to_hand(opponent_hand_card);
    }

    if args.connect {
        session.connect(user_id).expect("Connection failed");
        session.connect(opponent_id).expect("Connection failed");
    }
    session
}

#[derive(Clone, Debug)]
pub struct AdventureArgs {
    /// Coins the player currently has in this adventure
    pub current_coins: Coins,
    /// Coins the player will receive if they win this battle
    pub reward: Coins,
}

/// Arguments to [new_game]
#[derive(Clone, Debug)]
pub struct Args {
    /// Player whose turn it should be. Defaults to the `user_side` player.
    pub turn: Option<Side>,
    /// Mana available for the `user_side` player. Defaults to 999
    /// ([STARTING_MANA]).
    pub mana: ManaValue,
    /// Mana for the opponent of the `user_side` player. Defaults to 999
    /// ([STARTING_MANA]).
    pub opponent_mana: ManaValue,
    /// Actions available for the `turn` player. Defaults to 3.
    pub actions: ActionCount,
    /// Score for the `user_side` player. Defaults to 0.
    pub score: PointsValue,
    /// Score for the opponent of the `user_side` player. Defaults to 0.
    pub opponent_score: PointsValue,
    /// Starting size for the `user_side` player's hand, draw from the top of
    /// their deck. Hand will consist entirely of 'test spell' cards.
    /// Defaults to 0.
    pub hand_size: u64,
    /// Starting size for the opponent player's hand, draw from the top of their
    /// deck. Hand will consist entirely of 'test spell' cards. Defaults to
    /// 0.
    pub opponent_hand_size: u64,
    /// Card to be inserted into the `user_side` player's deck as the next draw.
    ///
    /// This card will be drawn when drawing randomly from the deck (as long as
    /// no known cards are placed on top of it) because the game is created with
    /// [GameConfiguration::deterministic] set to true.
    pub deck_top: Vec<CardName>,
    /// Card to be inserted into the opponent player's deck as the next draw.
    pub opponent_deck_top: Vec<CardName>,
    /// Card to be inserted into the `user_side` player's discard pile.
    pub discard: Option<CardName>,
    /// Card to be inserted into the opponent player's discard pile.
    pub opponent_discard: Option<CardName>,
    /// Sigils which start in play for the `user_side` player.
    pub sigils: Vec<CardName>,
    /// Sigils which start in play for the opponent player.
    pub opponent_sigils: Vec<CardName>,
    /// Set up an active raid within the created game using [ROOM_ID] as the
    /// target and [RAID_ID] as the ID.
    pub add_raid: bool,
    /// If false, will not attempt to automatically connect to this game.
    /// Defaults to true.
    pub connect: bool,
    /// If true, will configure the created game in scripted tutorial mode.
    pub tutorial: bool,
    /// Add an ongoing adventure to the database for this player
    pub adventure: Option<AdventureArgs>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            turn: None,
            mana: STARTING_MANA,
            opponent_mana: STARTING_MANA,
            actions: 3,
            score: 0,
            opponent_score: 0,
            hand_size: 0,
            opponent_hand_size: 0,
            deck_top: vec![],
            opponent_deck_top: vec![],
            discard: None,
            opponent_discard: None,
            sigils: vec![],
            opponent_sigils: vec![],
            add_raid: false,
            connect: true,
            tutorial: false,
            adventure: None,
        }
    }
}

fn set_deck_top(game: &mut GameState, side: Side, deck_top: Vec<CardName>) {
    for card in deck_top {
        let target_id = game
            .cards(side)
            .iter()
            .find(|c| c.position().kind() == CardPositionKind::DeckUnknown)
            .expect("No cards in deck")
            .id;
        test_session::overwrite_card(game, target_id, card);
        game.move_card_internal(target_id, CardPosition::DeckTop(side))
    }
}

fn set_discard_pile(game: &mut GameState, side: Side, discard: Option<CardName>) {
    if let Some(discard) = discard {
        let target_id = game
            .cards(side)
            .iter()
            .filter(|c| c.position().kind() == CardPositionKind::DeckUnknown)
            .last() // Take last to avoid overwriting deck top
            .expect("No cards in deck")
            .id;
        test_session::overwrite_card(game, target_id, discard);
        game.move_card_internal(target_id, CardPosition::DiscardPile(side));
        game.card_mut(target_id).turn_face_down();
    }
}

/// Creates an empty [TestSession]. Both provided [PlayerId]s are mapped to
/// empty data. If a game is requested for the session, it will receive the
/// provided [GameId].
pub fn new_session(game_id: GameId, user_id: PlayerId, opponent_id: PlayerId) -> TestSession {
    cards_all::initialize();

    let database = FakeDatabase {
        generated_game_id: Some(game_id),
        game: Mutex::new(None),
        players: Mutex::new(hashmap! {
            user_id => PlayerState {
                id: user_id,
                status: None,
                adventure: None,
                tutorial: TutorialData::default()
            },
            opponent_id => PlayerState {
                id: opponent_id,
                status: None,
                adventure: None,
                tutorial: TutorialData::default()
            }
        }),
    };

    TestSession::new(database, user_id, opponent_id)
}

fn create_mock_adventure(player_id: PlayerId, side: Side, args: AdventureArgs) -> AdventureState {
    let battle = TileEntity::Battle(BattleData {
        opponent_id: AIPlayer::NoAction,
        opponent_deck: decklists::canonical_deck(side.opponent()),
        opponent_name: "Opponent Name".to_string(),
        reward: args.reward,
        character: CharacterPreset::Overlord,
        character_facing: CharacterFacing::Down,
        region_to_reveal: 2,
    });
    let mut adventure = mock_adventure::create(
        AdventureId::new_from_u128(0),
        AdventureConfiguration {
            player_id,
            side,
            rng: Some(Xoshiro256StarStar::seed_from_u64(314159265358979323)),
        },
        decklists::canonical_deck(side),
        HashMap::new(),
        None,
        None,
        None,
        Some(battle),
    );
    adventure.visiting_position = Some(mock_adventure::BATTLE_POSITION);
    adventure.coins = args.current_coins;
    adventure
}
