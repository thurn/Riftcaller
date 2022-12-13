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

use core_ui::prelude::*;
use core_ui::style;
use data::adventure::DraftData;
use data::player_data::PlayerData;

use crate::full_screen_image_panel::FullScreenImagePanel;

pub struct DraftPanel<'a> {
    pub data: &'a DraftData,
}

impl<'a> DraftPanel<'a> {
    pub fn from_player(_player: &PlayerData) -> Self {
        todo!("")
    }
}

impl<'a> Component for DraftPanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImagePanel::new()
            .image(style::sprite("TPR/EnvironmentsHQ/mountain"))
            .content(Column::new("DraftPanel"))
            .build()
    }
}
