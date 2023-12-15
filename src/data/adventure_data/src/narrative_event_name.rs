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

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// Possible names of cards.
///
/// This enum is used to connect the state of a card to its game rules.
#[derive(
    PartialEq, Eq, Hash, Debug, Copy, Clone, Display, EnumString, Serialize, Deserialize, Sequence,
)]
pub enum NarrativeEventName {
    StormfeatherEagle,
}
