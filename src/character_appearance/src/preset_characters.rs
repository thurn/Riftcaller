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

use protos::spelldawn::{
    CharacterAppearance, CharacterAppearanceCustomization, CharacterAppearanceScaleGroup,
    CharacterScaleGroupName, CustomizationDataAddress, FlexColor,
};

pub fn overlord() -> CharacterAppearance {
    CharacterAppearance {
        body_color: color("EED0B5"),
        customizations: vec![
            customization("DanielThomas/2D Customizable Characters/Characters/Male/Data/Appearance/Mouth/Mouth Neutral.asset"),
            customization_with_color("DanielThomas/2D Customizable Characters/Characters/Male/Data/Appearance/Iris/Iris 1.asset", "8C8FD6"),
            customization_with_color("DanielThomas/2D Customizable Characters/Characters/Male/Data/Appearance/Eyebrows/Eyebrows 1.asset", "A49279"),
            customization_with_details("DanielThomas/2D Customizable Characters/Characters/Female/Data/Equipment/Headwear/Cap 3.asset", "54C338", 0, "FF8840"),
            customization_with_details("DanielThomas/2D Customizable Characters/Characters/Male/Data/Equipment/Tops/Layered Tunic.asset", "54C339", 0, "FFFFFF"),
            customization_with_details("DanielThomas/2D Customizable Characters/Characters/Male/Data/Equipment/Belts/Belt 2.asset", "AD7733", 1, "FFFFFF"),
            customization_with_color("DanielThomas/2D Customizable Characters/Characters/Male/Data/Equipment/Pants/Cloth Pants.asset", "675241"),
            customization_with_details("DanielThomas/2D Customizable Characters/Characters/Male/Data/Equipment/Footwear/Open Shoe.asset", "B28144", 2, "D1D1D1"),
            customization_with_color("DanielThomas/2D Customizable Characters/Characters/Male/Data/Equipment/Handwear/Gloves.asset", "B28144"),
            customization_with_color("DanielThomas/2D Customizable Characters/Characters/Male/Data/Appearance/Hair/Headwear Hair Short.asset", "C69455"),
            customization_with_details("DanielThomas/2D Customizable Characters/Characters/Male/Data/Appearance/Ears/Ears 2.asset", "FFFFFF", 0, "FFFFFF"),
            customization("DanielThomas/2D Customizable Characters/Characters/Male/Data/Appearance/Eyes/Eyes 2.asset"),
            customization_with_color("DanielThomas/2D Customizable Characters/Characters/Male/Data/Appearance/Facial Hair/Facial Hair 1.asset", "C69455"),
            customization_with_details("DanielThomas/2D Customizable Characters/Characters/Male/Data/Equipment/Weapons/Sword 1.asset", "FFFFFF", 0, "FFFFFF"),
            customization_with_details("DanielThomas/2D Customizable Characters/Characters/Male/Data/Equipment/Shields/Shield 1.asset", "E09B60", 0, "FFFFFF")
        ],
        scale_groups: vec![
            scale(CharacterScaleGroupName::Body, 1.024),
            scale(CharacterScaleGroupName::Head, 0.939),
            default_scale(CharacterScaleGroupName::Arms),
            default_scale(CharacterScaleGroupName::Hands),
            default_scale(CharacterScaleGroupName::Legs),
            default_scale(CharacterScaleGroupName::Feet),
            scale(CharacterScaleGroupName::Weapon, 0.9367525),
            default_scale(CharacterScaleGroupName::Shield)
        ]
    }
}

fn color(s: &str) -> Option<FlexColor> {
    let result = csscolorparser::parse(s).expect("Invalid color");
    Some(FlexColor {
        red: result.r as f32,
        green: result.g as f32,
        blue: result.b as f32,
        alpha: result.a as f32,
    })
}

fn customization(address: &str) -> CharacterAppearanceCustomization {
    CharacterAppearanceCustomization {
        data: Some(CustomizationDataAddress { address: address.to_string() }),
        color: None,
        detail_sprite_index: 0,
        detail_color: None,
    }
}

fn customization_with_color(address: &str, main_color: &str) -> CharacterAppearanceCustomization {
    CharacterAppearanceCustomization {
        data: Some(CustomizationDataAddress { address: address.to_string() }),
        color: color(main_color),
        detail_sprite_index: 0,
        detail_color: None,
    }
}

fn customization_with_details(
    address: &str,
    main_color: &str,
    index: u32,
    detail_color: &str,
) -> CharacterAppearanceCustomization {
    CharacterAppearanceCustomization {
        data: Some(CustomizationDataAddress { address: address.to_string() }),
        color: color(main_color),
        detail_sprite_index: index,
        detail_color: color(detail_color),
    }
}

fn scale(name: CharacterScaleGroupName, amount: f32) -> CharacterAppearanceScaleGroup {
    CharacterAppearanceScaleGroup { name: name.into(), scale: amount, width: 1.0, length: 1.0 }
}

fn default_scale(name: CharacterScaleGroupName) -> CharacterAppearanceScaleGroup {
    CharacterAppearanceScaleGroup { name: name.into(), scale: 1.0, width: 1.0, length: 1.0 }
}
