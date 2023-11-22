use game_data::text::{TextElement, TextToken};

/// Builds a named card trigger
pub fn named_trigger(name: TextToken, effect: Vec<TextElement>) -> Vec<TextElement> {
    vec![TextElement::NamedTrigger(name, effect)]
}
