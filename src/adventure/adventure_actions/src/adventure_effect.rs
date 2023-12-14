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

use adventure_data::adventure::AdventureState;
use adventure_data::adventure_effect::AdventureEffect;
use anyhow::Result;
use game_data::card_name::CardVariant;

pub fn apply(
    _state: &mut AdventureState,
    effect: AdventureEffect,
    _known_card: Option<CardVariant>,
) -> Result<()> {
    match effect {
        AdventureEffect::Draft(_) => {}
        _ => {}
    }
    Ok(())
}
