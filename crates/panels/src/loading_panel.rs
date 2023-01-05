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

use core_ui::full_screen_loading::FullScreenLoading;
use core_ui::prelude::*;
use panel_address::{Panel, PanelAddress};

/// A simple panel which displays a full-screen loading image
pub struct LoadingPanel {
    address: PanelAddress,
    image: String,
}

impl LoadingPanel {
    pub fn new(address: PanelAddress, image: impl Into<String>) -> Self {
        Self { address, image: image.into() }
    }
}

impl Panel for LoadingPanel {
    fn address(&self) -> PanelAddress {
        self.address
    }
}

impl Component for LoadingPanel {
    fn build(self) -> Option<Node> {
        FullScreenLoading::new(self.image).build()
    }
}
