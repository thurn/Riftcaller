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

use adventure_data::adventure::{BattleData, Coins, TileEntity};
use game_data::character_preset::{CharacterFacing, CharacterPreset};
use game_data::player_name::AIPlayer;
use game_data::primitives::Side;
use test_utils::client_interface::HasText;
use test_utils::test_adventure::{TestAdventure, TestConfig};

const BATTLE_REWARD: u32 = 250;

#[test]
fn test_open_battle_screen() {
    let mut adventure = TestAdventure::new(Side::Champion, config());
    adventure.visit_tile_with_character();
    assert!(adventure.interface.top_panel().has_text("Battle"));
}

#[test]
fn test_start_battle() {
    let mut adventure = TestAdventure::new(Side::Champion, config());
    adventure.visit_tile_with_character();
    assert_eq!("World", adventure.current_scene());
    adventure.click_on("Start");
    assert_eq!("Game", adventure.current_scene());
}

fn config() -> TestConfig {
    TestConfig {
        battle: Some(TileEntity::Battle(BattleData {
            opponent_id: AIPlayer::NoAction,
            opponent_deck: decklists::canonical_deck(Side::Overlord),
            opponent_name: "Opponent Name".to_string(),
            reward: Coins(BATTLE_REWARD),
            character: CharacterPreset::Overlord,
            character_facing: CharacterFacing::Down,
            region_to_reveal: 2,
        })),
        ..TestConfig::default()
    }
}
