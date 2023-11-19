use game_data::text::{TextElement, TextToken};

/// Builds a named card trigger
pub fn trigger_text(name: TextToken, effect: Vec<TextElement>) -> Vec<TextElement> {
    vec![TextElement::NamedTrigger(name, effect)]
}
