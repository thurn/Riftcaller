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

pub mod client_interface;
pub mod fake_database;
pub mod summarize;
pub mod test_adventure;
pub mod test_adventure_helpers;
pub mod test_game;
pub mod test_game_client;
pub mod test_helpers;
pub mod test_interface_helpers;
pub mod test_session;
pub mod test_session_builder;
pub mod test_session_helpers;
pub mod test_world_map;

pub use test_adventure_helpers::TestAdventureHelpers;
pub use test_game_client::CardNamesExt;
pub use test_interface_helpers::{Button, TestInterfaceHelpers};
pub use test_session_helpers::TestSessionHelpers;
