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

use adventure_data::adventure_effect_data::AdventureEffect;
use core_data::game_primitives::Side;
use test_utils::test_adventure::TestAdventure;
use test_utils::*;

#[test]
fn test_open_battle_screen() {
    let mut adventure = TestAdventure::new(Side::Riftcaller).build();

    let battle = adventure.insert_tile(AdventureEffect::Battle);
    adventure.visit_tile(battle);
    assert!(adventure.has_text("Battle"));
}

#[test]
fn test_start_battle() {
    let mut adventure = TestAdventure::new(Side::Riftcaller).build();

    let battle = adventure.insert_tile(AdventureEffect::Battle);
    adventure.visit_tile(battle);
    assert_eq!("World", adventure.client.current_scene());
    adventure.click(Button::StartBattle);
    assert_eq!("Game", adventure.client.current_scene());
}
