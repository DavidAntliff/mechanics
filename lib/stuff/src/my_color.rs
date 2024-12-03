use scarlet::prelude::*;

// For conversion from Scarlet to Bevy Color types - both are sRGB
pub struct MyColor(bevy::color::Color);

impl From<scarlet::color::RGBColor> for MyColor {
    fn from(value: RGBColor) -> Self {
        MyColor(bevy::color::Color::srgb(
            value.r as f32,
            value.g as f32,
            value.b as f32,
        ))
    }
}

impl From<MyColor> for bevy::color::Color {
    fn from(value: MyColor) -> Self {
        value.0
    }
}
