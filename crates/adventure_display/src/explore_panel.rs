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
use core_ui::safe_screen::SafeScreen;
use core_ui::style;
use protos::spelldawn::{FlexPosition, ImageScaleMode};

pub struct ExplorePanel {}

impl Component for ExplorePanel {
    fn build(self) -> Option<Node> {
        let background =
            style::sprite("Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/QuarterSize/Basic_window_big_transparent");
        SafeScreen::new()
            .content(
                Row::new("ExplorePanel")
                    .style(
                        Style::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::All, 16.px()),
                    )
                    .child(
                        Column::new("ExploreBackground").style(
                            Style::new()
                                .background_image(style::sprite("TPR/InfiniteEnvironments/meadow"))
                                .position_type(FlexPosition::Absolute)
                                .position(Edge::All, 4.px())
                                .background_image_scale_mode(ImageScaleMode::StretchToFill),
                        ),
                    )
                    .child(
                        Column::new("ExploreBorder").style(
                            Style::new()
                                .background_image(background)
                                .position_type(FlexPosition::Absolute)
                                .position(Edge::All, 0.px())
                                .background_image_scale_mode(ImageScaleMode::StretchToFill)
                                .image_slice(Edge::All, 128.px()),
                        ),
                    ),
            )
            .build()
    }
}
