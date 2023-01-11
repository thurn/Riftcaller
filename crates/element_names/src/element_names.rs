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

//! Unique identifiers for UI elements

use std::sync::atomic::{AtomicU64, Ordering};

use data::card_name::CardName;
use protos::spelldawn::element_selector::Selector;
use protos::spelldawn::ElementSelector;

static NEXT: AtomicU64 = AtomicU64::new(1);

/// Represents a globally unique identifier for a UI element.
#[derive(Clone, Copy, Debug)]
pub struct ElementName {
    tag: &'static str,
    count: u64,
}

/// Represents the element currently being dragged
#[derive(Clone, Copy, Debug)]
pub struct CurrentDraggable;

/// Represents the name of a temporary target element
#[derive(Clone, Copy, Debug)]
pub struct TargetName(pub ElementName);

impl ElementName {
    /// Creates a new element name. The 'tag' is used to facilitate debugging
    /// and does *not* need to be globally unique.
    pub fn new(tag: &'static str) -> Self {
        Self { tag, count: NEXT.fetch_add(1, Ordering::SeqCst) }
    }
}

impl From<ElementName> for String {
    fn from(name: ElementName) -> Self {
        format!("{}{}", name.tag, name.count)
    }
}

pub trait ElementNameSelector: Copy + Sized {
    fn selector(self) -> ElementSelector;
}

impl ElementNameSelector for ElementName {
    fn selector(self) -> ElementSelector {
        ElementSelector { selector: Some(Selector::ElementName(self.into())) }
    }
}

impl ElementNameSelector for CurrentDraggable {
    fn selector(self) -> ElementSelector {
        ElementSelector { selector: Some(Selector::DragIndicator(())) }
    }
}

impl ElementNameSelector for TargetName {
    fn selector(self) -> ElementSelector {
        ElementSelector { selector: Some(Selector::TargetElement(self.0.into())) }
    }
}

const fn global(tag: &'static str) -> ElementName {
    ElementName { tag, count: 0 }
}

pub static DECK_BUTTON: ElementName = global("DeckButton");

pub static MENU_BUTTON: ElementName = global("MenuButton");

pub static FEEDBACK_BUTTON: ElementName = global("FeedbackButton");

pub static CARD_LIST: ElementName = global("CardList");

pub static COLLECTION_BROWSER: ElementName = global("CollectionBrowser");

pub fn deck_card(name: CardName) -> ElementName {
    ElementName { tag: "DeckCard", count: name as u64 }
}

pub fn deck_card_slot(name: CardName) -> ElementName {
    ElementName { tag: "DeckCardSlot", count: name as u64 }
}

pub fn card_list_card_name(name: CardName) -> ElementName {
    ElementName { tag: "CardListCardName", count: name as u64 }
}

pub fn buy_card(name: CardName) -> ElementName {
    ElementName { tag: "BuyCard", count: name as u64 }
}
