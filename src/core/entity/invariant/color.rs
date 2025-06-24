use csscolorparser::Color;
use derive_getters::Getters;
use derive_more::{Debug, Display};
use derive_new::new;
use std::str::FromStr;

use super::IllegalConversion;

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Getters, new)]
#[display("RGB({r}, {g}, {b})")]
pub struct ThemeColor {
    r: u8,
    g: u8,
    b: u8,
}

impl FromStr for ThemeColor {
    type Err = IllegalConversion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [r, g, b, _] = Color::from_str(s)
            .map_err(|_| IllegalConversion {
                value: s.into(),
                dtype: "ThemeColor",
            })?
            .to_rgba8();
        Ok(Self { r, g, b })
    }
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_hex_color() {
        let c: ThemeColor = "#FFCC12".parse().unwrap();
        assert_eq!(c.r(), &255);
        assert_eq!(c.g(), &204);
        assert_eq!(c.b(), &18);
        assert_eq!(c.to_string(), "RGB(255, 204, 18)");
    }

    #[test]
    fn valid_named_color() {
        let c: ThemeColor = "rebeccapurple".parse().unwrap();
        assert_eq!(c.r(), &102);
        assert_eq!(c.g(), &51);
        assert_eq!(c.b(), &153);
    }

    #[test]
    fn valid_rgb_functional_color() {
        let c: ThemeColor = "rgb(10, 20, 30)".parse().unwrap();
        assert_eq!(c.r(), &10);
        assert_eq!(c.g(), &20);
        assert_eq!(c.b(), &30);
    }

    #[test]
    fn invalid_color_format() {
        let c = "not-a-color".parse::<ThemeColor>();
        assert!(c.is_err());
    }
}
