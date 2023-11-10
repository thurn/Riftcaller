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

//! Core design primitives

use game_data::special_effects::{EffectColor, TimedEffectData};
use protos::spelldawn::{Dimension, FlexColor, FontAddress};

use crate::style::DimensionExt;

const fn color(red: f32, green: f32, blue: f32, alpha: f32) -> FlexColor {
    FlexColor { red, green, blue, alpha }
}

// Converted from hex using https://www.tydac.ch/color/
pub const WHITE: FlexColor = color(1.0, 1.0, 1.0, 1.0);
pub const BLACK: FlexColor = color(0.0, 0.0, 0.0, 1.0);
pub const BLACK_ALPHA_25: FlexColor = color(0.0, 0.0, 0.0, 0.25);
pub const BLACK_ALPHA_50: FlexColor = color(0.0, 0.0, 0.0, 0.5);
pub const BLACK_ALPHA_75: FlexColor = color(0.0, 0.0, 0.0, 0.75);
pub const RED_100: FlexColor = color(1.0, 0.8, 0.82, 1.0);
pub const RED_500: FlexColor = color(0.96, 0.26, 0.21, 1.0);
pub const RED_600: FlexColor = color(0.9, 0.22, 0.21, 1.0);
pub const RED_700: FlexColor = color(0.83, 0.18, 0.18, 1.0);
pub const RED_800: FlexColor = color(0.78, 0.16, 0.16, 1.0);
pub const RED_900: FlexColor = color(0.72, 0.11, 0.11, 1.0);
pub const RED_900_ALPHA_75: FlexColor = color(0.72, 0.11, 0.11, 0.75);
pub const BLUE_500: FlexColor = color(0.13, 0.59, 0.95, 1.0);
pub const BLUE_700: FlexColor = color(0.1, 0.46, 0.82, 1.0);
pub const BLUE_900: FlexColor = color(0.05, 0.28, 0.63, 1.0);
pub const GREEN_500: FlexColor = color(0.3, 0.69, 0.31, 1.0);
pub const GREEN_700: FlexColor = color(0.22, 0.56, 0.24, 1.0);
pub const GREEN_900: FlexColor = color(0.11, 0.37, 0.13, 1.0);
pub const GREEN_900_ALPHA_75: FlexColor = color(0.11, 0.37, 0.13, 0.75);
pub const YELLOW_500: FlexColor = color(1.0, 0.92, 0.23, 1.0);
pub const YELLOW_700: FlexColor = color(0.98, 0.75, 0.18, 1.0);
pub const YELLOW_900: FlexColor = color(0.96, 0.5, 0.09, 1.0);
pub const PINK_500: FlexColor = color(0.91, 0.12, 0.39, 1.0);
pub const PINK_700: FlexColor = color(0.76, 0.09, 0.36, 1.0);
pub const PINK_900: FlexColor = color(0.53, 0.05, 0.31, 1.0);
pub const ORANGE_500: FlexColor = color(1.0, 0.6, 0.0, 1.0);
pub const ORANGE_700: FlexColor = color(0.96, 0.49, 0.0, 1.0);
pub const ORANGE_900: FlexColor = color(0.9, 0.32, 0.0, 1.0);
pub const GRAY_500: FlexColor = color(0.62, 0.62, 0.62, 1.0);
pub const GRAY_700: FlexColor = color(0.38, 0.38, 0.38, 1.0);
pub const GRAY_900: FlexColor = color(0.13, 0.13, 0.13, 1.0);
pub const PURPLE_500: FlexColor = color(0.40, 0.23, 0.72, 1.0);
pub const PURPLE_700: FlexColor = color(0.32, 0.18, 0.66, 1.0);
pub const PURPLE_900: FlexColor = color(0.19, 0.11, 0.57, 1.0);

pub const TEXT_OUTLINE: FlexColor = BLACK;
pub const OVERLAY_BORDER: FlexColor = BLACK;
pub const COIN_COUNT_BORDER: FlexColor = GRAY_500;
pub const DISABLED_BUTTON_TINT: FlexColor = BLACK_ALPHA_50;
pub const PLAY_CARD_BROWSER_OUTLINE: FlexColor = YELLOW_500;

/// Converts a [FlexColor] into a hex code representation.
pub fn as_hex(input: impl Into<FlexColor>) -> String {
    let color = input.into();
    format!(
        "#{:02X}{:02X}{:02X}",
        (color.red * 255.0).round() as i32,
        (color.green * 255.0).round() as i32,
        (color.blue * 255.0).round() as i32
    )
}

#[derive(Debug, Clone, Copy)]
pub enum BorderColor {
    Toast,
}

impl From<BorderColor> for FlexColor {
    fn from(color: BorderColor) -> Self {
        match color {
            BorderColor::Toast => BLACK,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BackgroundColor {
    CardInfo,
    PositiveCardInfo,
    NegativeCardInfo,
    DeckEditorPanel,
    CardCount,
    SafeAreaOverlay,
    BottomSheetOverlay,
    BottomSheetBackground,
    DeckCardVariantOverlay,
    TilePanelOverlay,
    CoinCountOverlay,
    MediaOverlay,
    Tooltip,
    Toast,
    SpeechBubble,
    GameInstructionsBackground,
    AccessedNote,
}

impl From<BackgroundColor> for FlexColor {
    fn from(color: BackgroundColor) -> Self {
        match color {
            BackgroundColor::CardInfo => BLACK_ALPHA_75,
            BackgroundColor::PositiveCardInfo => GREEN_900_ALPHA_75,
            BackgroundColor::NegativeCardInfo => RED_900_ALPHA_75,
            BackgroundColor::DeckEditorPanel => BLACK,
            BackgroundColor::CardCount => BLACK,
            BackgroundColor::SafeAreaOverlay => BLACK,
            BackgroundColor::BottomSheetOverlay => BLACK_ALPHA_75,
            BackgroundColor::BottomSheetBackground => ORANGE_900,
            BackgroundColor::DeckCardVariantOverlay => BLACK_ALPHA_50,
            BackgroundColor::TilePanelOverlay => BLACK_ALPHA_50,
            BackgroundColor::CoinCountOverlay => BLACK_ALPHA_50,
            BackgroundColor::MediaOverlay => BLACK_ALPHA_50,
            BackgroundColor::Tooltip => GREEN_700,
            BackgroundColor::Toast => BLACK,
            BackgroundColor::SpeechBubble => WHITE,
            BackgroundColor::GameInstructionsBackground => BLACK_ALPHA_75,
            BackgroundColor::AccessedNote => BLACK_ALPHA_75,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FontColor {
    PrimaryText,
    ButtonLabel,
    ButtonLabelDisabled,
    PanelTitle,
    NormalCardTitle,
    MortalCardTitle,
    InfernalCardTitle,
    AstralCardTitle,
    MultiResonanceCardTitle,
    PrismaticCardTitle,
    CardCost,
    CoinCount,
    Tooltip,
    Toast,
    SpeechBubble,
}

impl From<FontColor> for FlexColor {
    fn from(color: FontColor) -> Self {
        match color {
            FontColor::PrimaryText => WHITE,
            FontColor::ButtonLabel => WHITE,
            FontColor::ButtonLabelDisabled => GRAY_500,
            FontColor::PanelTitle => WHITE,
            FontColor::NormalCardTitle => BLACK,
            FontColor::MortalCardTitle => GREEN_700,
            FontColor::InfernalCardTitle => RED_600,
            FontColor::AstralCardTitle => BLUE_500,
            FontColor::MultiResonanceCardTitle => PURPLE_500,
            FontColor::PrismaticCardTitle => ORANGE_900,
            FontColor::CardCost => WHITE,
            FontColor::CoinCount => YELLOW_700,
            FontColor::Tooltip => WHITE,
            FontColor::Toast => WHITE,
            FontColor::SpeechBubble => BLACK,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FontSize {
    ButtonLabel,
    ButtonLabelTwoLines,
    ButtonIcon,
    PanelTitle,
    PromptContext,
    SupplementalInfo,
    Headline,
    Body,
    Meta,
    CardVariant,
    CardCount,
    CardCost,
    CoinCount,
    Toast,
    SchoolLabel,
    GameInstructionsText,
    GameInstructionsMetaText,
}

impl From<FontSize> for Dimension {
    fn from(size: FontSize) -> Self {
        (match size {
            FontSize::ButtonLabel => 32,
            FontSize::ButtonLabelTwoLines => 28,
            FontSize::ButtonIcon => 48,
            FontSize::PanelTitle => 48,
            FontSize::PromptContext => 48,
            FontSize::SupplementalInfo => 28,
            FontSize::Headline => 36,
            FontSize::Body => 28,
            FontSize::Meta => 20,
            FontSize::CardVariant => 28,
            FontSize::CardCount => 24,
            FontSize::CardCost => 36,
            FontSize::CoinCount => 54,
            FontSize::Toast => 24,
            FontSize::SchoolLabel => 32,
            FontSize::GameInstructionsText => 36,
            FontSize::GameInstructionsMetaText => 28,
        })
        .px()
        .into()
    }
}

fn roboto() -> FontAddress {
    FontAddress { address: "Fonts/Roboto.ttf".to_string() }
}

fn impact() -> FontAddress {
    FontAddress { address: "Fonts/Impact.ttf".to_string() }
}

fn bluu_next() -> FontAddress {
    FontAddress { address: "Fonts/BluuNext-Bold.otf".to_string() }
}

fn bona_nova() -> FontAddress {
    FontAddress { address: "Fonts/BonaNova.ttf".to_string() }
}

#[derive(Debug, Clone, Copy)]
pub enum Font {
    PrimaryText,
    PanelTitle,
    ButtonLabel,
    CardIcon,
    CardVariant,
}

impl From<Font> for FontAddress {
    fn from(font: Font) -> Self {
        match font {
            Font::PrimaryText => roboto(),
            Font::PanelTitle => bluu_next(),
            Font::ButtonLabel => roboto(),
            Font::CardIcon => impact(),
            Font::CardVariant => bona_nova(),
        }
    }
}

pub trait TimedEffectDataExt {
    fn effect_color(self, color: FlexColor) -> Self;
}

impl TimedEffectDataExt for TimedEffectData {
    fn effect_color(mut self, color: FlexColor) -> Self {
        self.effect_color = Some(EffectColor {
            red: color.red,
            green: color.green,
            blue: color.blue,
            alpha: color.alpha,
        });
        self
    }
}
