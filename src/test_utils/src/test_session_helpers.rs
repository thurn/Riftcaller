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
use game_data::primitives::{Lineage, Side};
use protos::spelldawn::client_action::Action;
use protos::spelldawn::{CardIdentifier, LevelUpRoomAction, SpendActionPointAction};
use server::server_data::GameResponseOutput;

use crate::test_session::TestSession;
use crate::{CLIENT_ROOM_ID, ROOM_ID};

pub enum Buttons {
    Summon,
    NoSummon,
    NoWeapon,
    Score,
    EndRaid,
    Unveil,
    NoUnveil,
}

pub trait TestSessionHelpers {
    /// Spends the `side` player's action points with no effect until they have
    /// no action points remaining.
    fn spend_actions_until_turn_over(&mut self, side: Side);

    /// Levels up the [CLIENT_ROOM_ID] room a specified number of `times`. If
    /// this requires multiple turns, spends the Champion turns doing
    /// nothing.
    ///
    /// NOTE that this may cause the Champion to draw cards for their turn.
    fn level_up_room_times(&mut self, times: u32);

    /// Must be invoked during the Overlord turn. Performs the following
    /// actions:
    /// - Plays a test Scheme card
    /// - Ends the Overlord turn
    /// - Initiates a raid on the [ROOM_ID] room
    /// - Summons the minion in the room
    ///
    /// NOTE: This causes the Champion player to draw a card for their turn!
    fn set_up_minion_combat(&mut self);

    /// Equivalent to [Self::set_up_minion_combat] which invokes an `action`
    /// function
    /// at the start of the Champion's turn.
    fn set_up_minion_combat_with_action(&mut self, action: impl FnOnce(&mut TestSession));

    /// Must be invoked during the Champion turn. Performs the following
    /// actions:
    ///
    /// - Ends the Champion turn
    /// - Plays a 3-1 scheme in the [ROOM_ID] room.
    /// - Plays the provided `card_name` minion into that room.
    /// - Plays the selected minion in the [ROOM_ID] room.
    /// - Ends the Overlord turn.
    ///
    /// Returns a tuple of (scheme_id, minion_id).
    ///
    /// WARNING: This causes both players to draw cards for their turns!
    fn setup_raid_target(&mut self, card_name: CardName) -> (CardIdentifier, CardIdentifier);

    /// Look for a button in the user interface and invoke its action.
    fn click(&mut self, buttons: Buttons) -> GameResponseOutput;

    /// Must be invoked during the Champion turn. Performs the following
    /// actions:
    ///
    /// - Performs all actions described in [Self::setup_raid_target], creating
    ///   a minion of the indicated [Lineage] with `MINION_HEALTH` health.
    /// - Initiates a raid on the [ROOM_ID] room.
    /// - Summons the minion
    /// - Clicks on the button with text matching `name` in order to fire weapon
    ///   abilities.
    ///
    /// WARNING: This causes both players to draw cards for their turns!
    fn fire_weapon_combat_abilities(&mut self, lineage: Lineage, name: CardName);
}

impl TestSessionHelpers for TestSession {
    fn spend_actions_until_turn_over(&mut self, side: Side) {
        let id = self.player_id_for_side(side);
        while self.player(id).this_player.actions() > 0 {
            self.perform(Action::SpendActionPoint(SpendActionPointAction {}), id);
        }
    }

    fn level_up_room_times(&mut self, times: u32) {
        let mut levels = 0;
        let overlord_id = self.player_id_for_side(Side::Overlord);

        loop {
            while self.player(overlord_id).this_player.actions() > 0 {
                self.perform(
                    Action::LevelUpRoom(LevelUpRoomAction { room_id: CLIENT_ROOM_ID.into() }),
                    overlord_id,
                );
                levels += 1;

                if levels == times {
                    return;
                }
            }

            assert!(self.dawn());
            self.spend_actions_until_turn_over(Side::Champion);
            assert!(self.dusk());
        }
    }

    fn set_up_minion_combat(&mut self) {
        self.set_up_minion_combat_with_action(|_| {});
    }

    fn set_up_minion_combat_with_action(&mut self, action: impl FnOnce(&mut TestSession)) {
        self.create_and_play(CardName::TestScheme3_15);
        self.spend_actions_until_turn_over(Side::Overlord);
        assert!(self.dawn());
        action(self);
        self.initiate_raid(ROOM_ID);
        self.click(Buttons::Summon);
    }

    fn setup_raid_target(&mut self, card_name: CardName) -> (CardIdentifier, CardIdentifier) {
        self.spend_actions_until_turn_over(Side::Champion);
        assert!(self.dusk());
        let scheme_id = self.create_and_play(CardName::TestScheme3_15);
        let minion_id = self.create_and_play(card_name);
        self.spend_actions_until_turn_over(Side::Overlord);
        assert!(self.dawn());
        (scheme_id, minion_id)
    }

    fn click(&mut self, buttons: Buttons) -> GameResponseOutput {
        let (text, side) = match buttons {
            Buttons::Summon => ("Summon", Side::Overlord),
            Buttons::NoSummon => ("Pass", Side::Overlord),
            Buttons::NoWeapon => ("Continue", Side::Champion),
            Buttons::Score => ("Score", Side::Champion),
            Buttons::EndRaid => ("End Raid", Side::Champion),
            Buttons::Unveil => ("Unveil", Side::Overlord),
            Buttons::NoUnveil => ("Continue", Side::Overlord),
        };

        self.click_on(self.player_id_for_side(side), text)
    }

    fn fire_weapon_combat_abilities(&mut self, lineage: Lineage, name: CardName) {
        self.setup_raid_target(crate::test_helpers::minion_for_lineage(lineage));
        self.initiate_raid(ROOM_ID);
        self.click(Buttons::Summon);
        self.click_on(self.player_id_for_side(Side::Champion), name.displayed_name());
    }
}
