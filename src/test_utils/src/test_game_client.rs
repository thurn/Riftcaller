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

use std::cmp::Ordering;
use std::collections::HashMap;

use adapters;
use core_data::game_primitives::{ActionCount, CardId, ManaValue, PointsValue, RoomId, Side};
use game_data::card_name::{CardName, CardVariant};
use game_data::card_state::CardState;
use game_data::game_state::GameState;
use game_data::player_name::PlayerId;
use protos::riftcaller::card_targeting::Targeting;
use protos::riftcaller::game_command::Command;
use protos::riftcaller::game_object_identifier::Id;
use protos::riftcaller::object_position::Position;
use protos::riftcaller::tutorial_effect::TutorialEffectType;
use protos::riftcaller::{
    ArrowTargetRoom, CardIdentifier, CardView, ClientItemLocation, ClientRoomLocation, CommandList,
    GameMessageType, GameObjectIdentifier, NoTargeting, ObjectPosition, ObjectPositionBrowser,
    ObjectPositionCharacter, ObjectPositionDeck, ObjectPositionDiscardPile, ObjectPositionHand,
    ObjectPositionItem, ObjectPositionRaid, ObjectPositionRevealedCards, ObjectPositionRiftcallers,
    ObjectPositionRoom, PlayInRoom, PlayerName, PlayerView, RevealedCardView,
    RevealedCardsBrowserSize, RoomIdentifier,
};
use rules::dispatch;

use crate::client_interface::{ClientInterface, HasText};
use crate::test_world_map::TestWorldMap;

/// Overwrites the card with ID `card_id` in `game` to be a new card with the
/// provided `card_variant`.
pub fn overwrite_card(game: &mut GameState, card_id: CardId, card_variant: CardVariant) {
    let card = game.card(card_id);
    let mut state = CardState::new(card_id, card_variant);
    state.set_position_internal(card.sorting_key, card.position());
    *game.card_mut(card_id) = state;

    // Our delegate cache logic assumes the set of card names in a game will not
    // change while the game is in progress, so we need to delete the cache.
    dispatch::populate_delegate_cache(game);
}

/// Returns the [Side] player who owns the [CardName] card
pub fn side_for_card_name(name: CardName) -> Side {
    rules::get(CardVariant::standard(name)).side
}

/// Represents a user client connected to a test game
#[derive(Clone)]
pub struct TestGameClient {
    pub id: PlayerId,
    pub data: ClientGameData,
    /// A player's view of *their own* player state.
    pub this_player: ClientPlayer,
    /// A player's view of *their opponent's* player state.
    pub other_player: ClientPlayer,
    pub interface: ClientInterface,
    pub cards: ClientCards,
    pub history: Vec<Command>,
    pub map: TestWorldMap,
    current_scene: Option<String>,
}

impl TestGameClient {
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            data: ClientGameData::default(),
            this_player: ClientPlayer::new(PlayerName::User),
            other_player: ClientPlayer::new(PlayerName::Opponent),
            interface: ClientInterface::default(),
            cards: ClientCards { player_id: id, card_map: HashMap::default() },
            history: vec![],
            map: TestWorldMap::default(),
            current_scene: None,
        }
    }

    pub fn get_card(&self, id: CardIdentifier) -> &ClientCard {
        self.cards.get(id)
    }

    pub fn current_scene(&self) -> &str {
        self.current_scene.as_ref().expect("No LoadSceneCommand received")
    }

    pub fn handle_command_list(&mut self, list: CommandList) {
        for command in &list.commands {
            let c = command.command.as_ref().expect("command");
            self.handle_command(c);
        }
    }

    pub fn handle_command(&mut self, command: &Command) {
        self.data.update(command.clone());
        self.this_player.update(command.clone());
        self.other_player.update(command.clone());
        self.interface.update(command.clone());
        self.cards.update(command.clone());
        self.history.push(command.clone());
        self.map.update(command.clone());

        if let Command::LoadScene(s) = command {
            self.current_scene = Some(s.scene_name.clone());
        }
    }
}

/// Simulated game state in an ongoing TestSession
#[derive(Clone, Default)]
pub struct ClientGameData {
    raid_active: Option<bool>,
    object_positions: HashMap<GameObjectIdentifier, ObjectPosition>,
    last_message: Option<GameMessageType>,
    tutorial_effects: Vec<TutorialEffectType>,
}

impl ClientGameData {
    pub fn raid_active(&self) -> bool {
        self.raid_active.expect("raid_active")
    }

    /// Returns the position of the `id` object along with its index within its
    /// position list
    pub fn object_index_position(&self, id: Id) -> (u32, Position) {
        let position = self
            .object_positions
            .get(&GameObjectIdentifier { id: Some(id) })
            .unwrap_or_else(|| panic!("No position available for {id:?}"))
            .clone()
            .position
            .expect("position");
        let mut positions = self
            .object_positions
            .iter()
            .filter(|(_, p)| p.position.as_ref().expect("position") == &position)
            .collect::<Vec<_>>();
        positions.sort_by_key(|(_, p)| (p.sorting_key, p.sorting_subkey));
        let index = positions
            .iter()
            .position(|(object_id, _)| object_id.id.as_ref().expect("id") == &id)
            .expect("index");

        (index as u32, position)
    }

    /// Returns the position of the `id` object
    pub fn object_position(&self, id: Id) -> Position {
        self.object_index_position(id).1
    }

    /// Returns the last-seen `GameMessage`.
    pub fn last_message(&self) -> GameMessageType {
        self.last_message.expect("Game Message")
    }

    /// Returns the text of the first tutorial toast which will be shown to the
    /// user, or panics if no toast is showing.
    pub fn toast(&self) -> String {
        self.tutorial_effects
            .iter()
            .filter_map(|effect| match effect {
                TutorialEffectType::ArrowBubble(_) => None,
                TutorialEffectType::ShowToast(toast) => Some(toast.node.clone().expect("Node")),
            })
            .next()
            .expect("Toast")
            .all_text()
    }

    fn update(&mut self, command: Command) {
        match command {
            Command::UpdateGameView(update_game) => {
                let game = update_game.game.as_ref().unwrap();
                self.raid_active = Some(game.raid_active);
                for card in &game.cards {
                    self.object_positions
                        .insert(card_object_id(card.card_id), card.card_position.clone().unwrap());
                }

                let non_card = game.game_object_positions.as_ref().unwrap();
                self.insert_position(deck_id(PlayerName::User), &non_card.user_deck);
                self.insert_position(deck_id(PlayerName::Opponent), &non_card.opponent_deck);
                self.insert_position(character_id(PlayerName::User), &non_card.user_character);
                self.insert_position(
                    character_id(PlayerName::Opponent),
                    &non_card.opponent_character,
                );
                self.insert_position(discard_id(PlayerName::User), &non_card.user_discard);
                self.insert_position(discard_id(PlayerName::Opponent), &non_card.opponent_deck);
                self.tutorial_effects = game
                    .tutorial_effects
                    .clone()
                    .iter()
                    .filter_map(|e| e.tutorial_effect_type.clone())
                    .collect();
            }
            Command::MoveGameObjects(move_objects) => {
                for move_object in move_objects.moves {
                    let p = move_object.position.as_ref().expect("ObjectPosition").clone();
                    self.object_positions.insert(move_object.id.expect("id"), p);
                }
            }
            Command::DisplayGameMessage(display_message) => {
                self.last_message = GameMessageType::from_i32(display_message.message_type);
            }
            Command::CreateTokenCard(create_token) => {
                let card = create_token.card.as_ref().expect("card");
                self.object_positions.insert(
                    card_object_id(card.card_id),
                    card.card_position.clone().expect("position"),
                );
            }
            _ => {}
        }
    }

    fn insert_position(&mut self, id: GameObjectIdentifier, position: &Option<ObjectPosition>) {
        self.object_positions.insert(id, position.clone().expect("position"));
    }
}

fn card_object_id(id: Option<CardIdentifier>) -> GameObjectIdentifier {
    GameObjectIdentifier { id: Some(Id::CardId(id.expect("card_id"))) }
}

fn deck_id(name: PlayerName) -> GameObjectIdentifier {
    GameObjectIdentifier { id: Some(Id::Deck(name as i32)) }
}

fn character_id(name: PlayerName) -> GameObjectIdentifier {
    GameObjectIdentifier { id: Some(Id::Character(name as i32)) }
}

fn discard_id(name: PlayerName) -> GameObjectIdentifier {
    GameObjectIdentifier { id: Some(Id::DiscardPile(name as i32)) }
}

/// Simulated player state in an ongoing TestSession
#[derive(Debug, Clone)]
pub struct ClientPlayer {
    name: PlayerName,
    mana: Option<ManaValue>,
    bonus_mana: Option<ManaValue>,
    actions: Option<ActionCount>,
    score: Option<PointsValue>,
    can_take_action: Option<bool>,
}

impl ClientPlayer {
    fn new(name: PlayerName) -> Self {
        Self {
            name,
            mana: None,
            bonus_mana: None,
            actions: None,
            score: None,
            can_take_action: None,
        }
    }

    pub fn mana(&self) -> ManaValue {
        self.mana.expect("Mana")
    }

    pub fn bonus_mana(&self) -> ManaValue {
        self.bonus_mana.expect("BonusMana")
    }

    pub fn actions(&self) -> ActionCount {
        self.actions.expect("Actions")
    }

    pub fn score(&self) -> PointsValue {
        self.score.expect("Points")
    }

    pub fn can_take_action(&self) -> bool {
        self.can_take_action.expect("can_take_action")
    }

    fn update(&mut self, command: Command) {
        if let Command::UpdateGameView(update) = command {
            self.update_with_player(if self.name == PlayerName::User {
                update.game.unwrap().user
            } else {
                update.game.unwrap().opponent
            });
        }
    }

    fn update_with_player(&mut self, player: Option<PlayerView>) {
        if let Some(p) = player {
            self.mana = Some(p.mana.clone().expect("mana").base_mana);
            self.bonus_mana = Some(p.mana.clone().expect("mana").bonus_mana);
            self.actions = Some(p.action_tracker.clone().expect("actions").available_action_count);
            self.score = Some(p.score.clone().expect("score").score);
            self.can_take_action = Some(p.can_take_action);
        }
    }
}

pub trait CardNamesExt {
    fn names(&self) -> Vec<String>;

    fn ids(&self) -> Vec<CardIdentifier>;

    fn real_cards(&self) -> Self;

    fn token_cards(&self) -> Self;

    fn find_card(&self, name: CardName) -> &ClientCard;

    fn find_ability_card(&self, name: CardName) -> &ClientCard;

    fn find_card_id(&self, name: CardName) -> CardIdentifier;

    fn contains_card(&self, name: CardName) -> bool;

    fn curse_count(&self) -> usize;

    fn wound_count(&self) -> usize;

    fn leyline_count(&self) -> usize;
}

impl CardNamesExt for Vec<&ClientCard> {
    fn names(&self) -> Vec<String> {
        let mut result = self
            .iter()
            .map(|c| c.title_option().unwrap_or_else(|| test_constants::HIDDEN_CARD.to_string()))
            .collect::<Vec<_>>();
        result.sort();
        result
    }

    fn ids(&self) -> Vec<CardIdentifier> {
        let mut result = self.iter().map(|c| c.id()).collect::<Vec<_>>();
        result.sort();
        result
    }

    fn real_cards(&self) -> Self {
        self.clone()
            .into_iter()
            .filter(move |c| {
                c.id.expect("id").ability_id.is_none() && c.id.expect("id").game_action.is_none()
            })
            .collect()
    }

    fn token_cards(&self) -> Self {
        self.clone()
            .into_iter()
            .filter(move |c| {
                c.id.expect("id").ability_id.is_some() || c.id.expect("id").game_action.is_some()
            })
            .collect()
    }

    fn find_card(&self, name: CardName) -> &ClientCard {
        self.iter().find(|c| c.title() == name.displayed_name()).expect("Card not found")
    }

    fn find_ability_card(&self, name: CardName) -> &ClientCard {
        self.iter().find(|c| c.title().contains(&name.displayed_name())).expect("Card not found")
    }

    fn find_card_id(&self, name: CardName) -> CardIdentifier {
        self.find_card(name).id()
    }

    fn contains_card(&self, name: CardName) -> bool {
        self.iter().any(|c| c.title() == name.displayed_name())
    }

    fn curse_count(&self) -> usize {
        count_with_name(self, "Curse")
    }

    fn wound_count(&self) -> usize {
        count_with_name(self, "Wound")
    }

    fn leyline_count(&self) -> usize {
        count_with_name(self, "Leyline")
    }
}

fn count_with_name(cards: &[&ClientCard], name: &'static str) -> usize {
    cards.iter().filter(|c| c.title() == name).count()
}

/// Simulated card state in an ongoing TestSession
#[derive(Debug, Clone)]
pub struct ClientCards {
    pub player_id: PlayerId,
    pub card_map: HashMap<CardIdentifier, ClientCard>,
}

impl ClientCards {
    pub fn get(&self, card_id: CardIdentifier) -> &ClientCard {
        self.card_map.get(&card_id).unwrap_or_else(|| panic!("Card not found: {card_id:?}"))
    }

    pub fn in_position_iterator(&self, position: Position) -> impl Iterator<Item = &ClientCard> {
        self.card_map.values().filter(move |c| c.position() == position)
    }

    /// Returns an iterator over the cards in a given [Position] in an arbitrary
    /// order.
    fn in_position(&self, position: Position) -> Vec<&ClientCard> {
        self.in_position_iterator(position).collect()
    }

    pub fn hand(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Hand(ObjectPositionHand { owner: PlayerName::User.into() }))
    }

    pub fn opponent_hand(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Hand(ObjectPositionHand { owner: PlayerName::Opponent.into() }))
    }

    pub fn browser(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Browser(ObjectPositionBrowser {}))
    }

    pub fn raid_display(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Raid(ObjectPositionRaid {}))
    }

    /// Returns cards currently displayed in the revealed cards area
    pub fn revealed_cards_browser(&self) -> Vec<&ClientCard> {
        self.in_position_iterator(Position::Revealed(ObjectPositionRevealedCards {
            size: RevealedCardsBrowserSize::Small as i32,
        }))
        .chain(self.in_position_iterator(Position::Revealed(ObjectPositionRevealedCards {
            size: RevealedCardsBrowserSize::Large as i32,
        })))
        .collect()
    }

    pub fn discard_pile(&self) -> Vec<&ClientCard> {
        self.in_position(Position::DiscardPile(ObjectPositionDiscardPile {
            owner: PlayerName::User.into(),
        }))
    }

    pub fn opponent_discard_pile(&self) -> Vec<&ClientCard> {
        self.in_position(Position::DiscardPile(ObjectPositionDiscardPile {
            owner: PlayerName::Opponent.into(),
        }))
    }

    pub fn deck_top(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Deck(ObjectPositionDeck { owner: PlayerName::User.into() }))
    }

    pub fn opponent_deck_top(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Deck(ObjectPositionDeck { owner: PlayerName::Opponent.into() }))
    }

    pub fn artifacts(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Item(ObjectPositionItem {
            item_location: ClientItemLocation::Left.into(),
        }))
    }

    pub fn evocations_and_allies(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Item(ObjectPositionItem {
            item_location: ClientItemLocation::Right as i32,
        }))
    }

    pub fn room_defenders(&self, room_id: RoomId) -> Vec<&ClientCard> {
        self.in_position(Position::Room(ObjectPositionRoom {
            room_id: adapters::room_identifier(room_id),
            room_location: ClientRoomLocation::Front.into(),
        }))
    }

    pub fn room_occupants(&self, room_id: RoomId) -> Vec<&ClientCard> {
        self.in_position(Position::Room(ObjectPositionRoom {
            room_id: adapters::room_identifier(room_id),
            room_location: ClientRoomLocation::Back.into(),
        }))
    }

    pub fn display_shelf(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Riftcaller(ObjectPositionRiftcallers {
            owner: PlayerName::User.into(),
        }))
    }

    pub fn opponent_display_shelf(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Riftcaller(ObjectPositionRiftcallers {
            owner: PlayerName::Opponent.into(),
        }))
    }

    pub fn score_area(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Character(ObjectPositionCharacter {
            owner: PlayerName::User.into(),
        }))
    }

    pub fn opponent_score_area(&self) -> Vec<&ClientCard> {
        self.in_position(Position::Character(ObjectPositionCharacter {
            owner: PlayerName::Opponent.into(),
        }))
    }

    /// Looks for the ID of card in the user's hand with a given name. Panics if
    /// no such card can be found.
    pub fn find_in_hand(&self, card: CardVariant) -> CardIdentifier {
        self.hand()
            .into_iter()
            .find(|c| c.title() == card.displayed_name())
            .expect("Card in hand")
            .id()
    }

    fn update(&mut self, command: Command) {
        match command {
            Command::UpdateGameView(update_game) => {
                let game = update_game.game.as_ref().unwrap();
                self.card_map.clear();
                for card in &game.cards {
                    self.card_map.insert(card.card_id.expect("card_id"), ClientCard::new(card));
                }
            }
            Command::MoveGameObjects(move_objects) => {
                for move_object in move_objects.moves {
                    let p = move_object.position.as_ref().expect("ObjectPosition").clone();
                    let id = match move_object.id.expect("id").id.expect("id") {
                        Id::CardId(identifier) => identifier,
                        _ => panic!("Expected CardId"),
                    };
                    self.card_map.get_mut(&id).unwrap().set_position(p);
                }
            }
            Command::CreateTokenCard(create_token) => {
                let card = create_token.card.as_ref().expect("card");
                self.card_map.insert(card.card_id.expect("card_id"), ClientCard::new(card));
            }
            _ => {}
        }
    }
}

/// Simulated state of a specific card
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientCard {
    id: Option<CardIdentifier>,
    title: Option<String>,
    rules_text: Option<String>,
    position: Option<ObjectPosition>,
    revealed_to_me: Option<bool>,
    is_face_up: Option<bool>,
    can_play: Option<bool>,
    valid_rooms: Option<Vec<RoomIdentifier>>,
    arena_icon: Option<String>,
    top_left_icon: Option<String>,
    top_right_icon: Option<String>,
    bottom_left_icon: Option<String>,
    bottom_right_icon: Option<String>,
}

impl ClientCard {
    pub fn id(&self) -> CardIdentifier {
        self.id.expect("card_id")
    }

    /// Returns the game object position for this card
    pub fn position(&self) -> Position {
        self.position.clone().expect("CardPosition").position.expect("Position")
    }

    /// Returns the user-visible title for this card. Panics if no title is
    /// available.
    pub fn title(&self) -> String {
        self.title_option().expect("No card title found")
    }

    /// Returns a copy of the user-visible title for this card, if one is
    /// available
    pub fn title_option(&self) -> Option<String> {
        self.title.clone()
    }

    pub fn rules_text(&self) -> String {
        self.rules_text.clone().expect("No rules text found")
    }

    pub fn revealed_to_me(&self) -> bool {
        self.revealed_to_me.expect("revealed_to_me")
    }

    pub fn is_face_up(&self) -> bool {
        self.is_face_up.expect("is_face_up")
    }

    pub fn can_play(&self) -> bool {
        self.can_play.expect("can_play")
    }

    pub fn valid_rooms(&self) -> Vec<RoomIdentifier> {
        self.valid_rooms.as_ref().expect("valid_rooms").clone()
    }

    pub fn arena_icon(&self) -> String {
        self.arena_icon.clone().expect("arena_icon")
    }

    pub fn top_left_icon(&self) -> String {
        self.top_left_icon.clone().expect("top_left_icon")
    }

    pub fn top_right_icon(&self) -> String {
        self.top_right_icon.clone().expect("top_right_icon")
    }

    pub fn bottom_left_icon(&self) -> String {
        self.bottom_left_icon.clone().expect("bottom_left_icon")
    }

    pub fn bottom_right_icon(&self) -> String {
        self.bottom_right_icon.clone().expect("bottom_right_icon")
    }

    pub fn attack_icon(&self) -> String {
        self.bottom_right_icon()
    }

    pub fn set_position(&mut self, position: ObjectPosition) {
        self.position = Some(position);
    }

    fn new(view: &CardView) -> Self {
        let mut result = Self::default();
        result.update(view);
        result
    }

    fn update(&mut self, view: &CardView) {
        self.id = view.card_id;
        self.position = view.card_position.clone();
        self.revealed_to_me = Some(view.revealed_to_viewer);
        self.is_face_up = Some(view.is_face_up);
        if let Some(revealed) = &view.revealed_card {
            self.update_revealed_card(revealed);
        }

        self.arena_icon = card_icon(view, |v| v.card_icons?.arena_icon?.text);
        self.top_left_icon = card_icon(view, |v| v.card_icons?.top_left_icon?.text);
        self.top_right_icon = card_icon(view, |v| v.card_icons?.top_right_icon?.text);
        self.bottom_left_icon = card_icon(view, |v| v.card_icons?.bottom_left_icon?.text);
        self.bottom_right_icon = card_icon(view, |v| v.card_icons?.bottom_right_icon?.text);
    }

    fn update_revealed_card(&mut self, revealed: &RevealedCardView) {
        let targets = {
            || {
                Some(match revealed.targeting.as_ref()?.targeting.as_ref()? {
                    Targeting::NoTargeting(NoTargeting { can_play }) => (*can_play, vec![]),
                    Targeting::PlayInRoom(PlayInRoom { valid_rooms }) => {
                        (!valid_rooms.is_empty(), valid_rooms.clone())
                    }
                    Targeting::ArrowTargetRoom(ArrowTargetRoom { valid_rooms, .. }) => {
                        (!valid_rooms.is_empty(), valid_rooms.clone())
                    }
                })
            }
        }();
        if let Some((can_play, valid_rooms)) = targets {
            self.can_play = Some(can_play);
            self.valid_rooms =
                Some(valid_rooms.iter().map(|i| RoomIdentifier::from_i32(*i).unwrap()).collect())
        }

        if let Some(title) = revealed.clone().title.map(|title| title.text) {
            self.title = Some(title);
        }

        self.rules_text = revealed.rules_text.as_ref().map(|r| r.text.clone())
    }
}

fn card_icon(view: &CardView, function: impl Fn(CardView) -> Option<String>) -> Option<String> {
    function(view.clone())
}

impl PartialOrd for ClientCard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.position.as_ref()?.sorting_key.partial_cmp(&other.position.as_ref()?.sorting_key)
    }
}
