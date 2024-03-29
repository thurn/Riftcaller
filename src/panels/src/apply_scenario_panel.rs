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

use core_ui::action_builder::ActionBuilder;
use core_ui::button::Button;
use core_ui::list_cell::ListCell;
use core_ui::panel_window::PanelWindow;
use core_ui::prelude::*;
use core_ui::scroll_view::ScrollView;
use panel_address::{Panel, PanelAddress, ScenarioKind, StandardPanel};
use user_action_data::{DebugAction, DebugAdventureScenario, DebugScenario, UserAction};

#[derive(Debug)]
pub struct ApplyScenarioPanel {
    kind: ScenarioKind,
}

impl ApplyScenarioPanel {
    pub fn new(kind: ScenarioKind) -> Self {
        Self { kind }
    }
}

impl Panel for ApplyScenarioPanel {
    fn address(&self) -> PanelAddress {
        PanelAddress::StandardPanel(StandardPanel::ApplyScenario(self.kind))
    }
}

impl Component for ApplyScenarioPanel {
    fn build(self) -> Option<Node> {
        PanelWindow::new(self.address(), 1200.px(), 900.px())
            .title("Apply Scenario")
            .show_close_button(true)
            .content(
                ScrollView::new("Scenario List")
                    .style(Style::new().margin(Edge::Vertical, 16.px()).flex_grow(1.0))
                    .child_nodes(match self.kind {
                        ScenarioKind::Game => enum_iterator::all::<DebugScenario>()
                            .map(|scenario| {
                                ListCell::new(scenario.displayed_name())
                                    .button(
                                        Button::new("Apply").action(
                                            ActionBuilder::new()
                                                .action(UserAction::Debug(
                                                    DebugAction::ApplyScenario(scenario),
                                                ))
                                                .update(self.close()),
                                        ),
                                    )
                                    .build()
                            })
                            .collect::<Vec<_>>()
                            .into_iter(),
                        ScenarioKind::Adventure => enum_iterator::all::<DebugAdventureScenario>()
                            .map(|scenario| {
                                ListCell::new(scenario.displayed_name())
                                    .button(
                                        Button::new("Apply").action(
                                            ActionBuilder::new()
                                                .action(UserAction::Debug(
                                                    DebugAction::ApplyAdventureScenario(scenario),
                                                ))
                                                .update(self.close()),
                                        ),
                                    )
                                    .build()
                            })
                            .collect::<Vec<_>>()
                            .into_iter(),
                    }),
            )
            .build()
    }
}
