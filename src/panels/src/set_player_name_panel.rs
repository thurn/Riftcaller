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

use core_ui::button::Button;
use core_ui::list_cell::ListCell;
use core_ui::panel_window::PanelWindow;
use core_ui::prelude::*;
use core_ui::scroll_view::ScrollView;
use game_data::player_name::NamedPlayer;
use game_data::primitives::Side;
use panel_address::{Panel, PanelAddress, StandardPanel};
use user_action_data::DebugAction;

#[derive(Debug)]
pub struct SetPlayerNamePanel {
    side: Side,
}

impl SetPlayerNamePanel {
    pub fn new(side: Side) -> Self {
        Self { side }
    }
}

impl Panel for SetPlayerNamePanel {
    fn address(&self) -> PanelAddress {
        StandardPanel::SetPlayerName(self.side).into()
    }
}

impl Component for SetPlayerNamePanel {
    fn build(self) -> Option<Node> {
        PanelWindow::new(self.address(), 1024.px(), 600.px())
            .title("Set Opponent")
            .show_close_button(true)
            .content(
                ScrollView::new("Opponent List")
                    .style(Style::new().margin(Edge::Vertical, 16.px()).flex_grow(1.0))
                    .children(enum_iterator::all::<NamedPlayer>().map(|n| {
                        ListCell::new(n.displayed_name()).button(
                            Button::new("Use").action(DebugAction::SetNamedPlayer(self.side, n)),
                        )
                    })),
            )
            .build()
    }
}
