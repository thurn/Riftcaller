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

use game_data::delegates::{Delegate, EventDelegate, MutationFn, RaidEvent};
use game_data::primitives::RoomId;

/// A delegate which fires when a card is face up & in play when a raid on the
/// sanctum ends in success
pub fn on_sanctum_access(mutation: MutationFn<RaidEvent>) -> Delegate {
    Delegate::RaidSuccess(EventDelegate {
        requirement: |g, s, raid| {
            crate::face_up_in_play(g, s, raid) && raid.target == RoomId::Sanctum
        },
        mutation,
    })
}
