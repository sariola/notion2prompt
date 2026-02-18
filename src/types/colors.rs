use super::ValidationError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Type-safe color enum instead of strings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum Color {
    #[default]
    Default,
    Gray,
    Brown,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Pink,
    LightGray,
    LightBrown,
    LightRed,
    LightOrange,
    LightYellow,
    LightGreen,
    LightBlue,
    LightPurple,
    LightPink,
}

impl std::str::FromStr for Color {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(Color::Default),
            "gray" => Ok(Color::Gray),
            "brown" => Ok(Color::Brown),
            "red" => Ok(Color::Red),
            "orange" => Ok(Color::Orange),
            "yellow" => Ok(Color::Yellow),
            "green" => Ok(Color::Green),
            "blue" => Ok(Color::Blue),
            "purple" => Ok(Color::Purple),
            "pink" => Ok(Color::Pink),
            "gray_background" | "light_gray" => Ok(Color::LightGray),
            "brown_background" | "light_brown" => Ok(Color::LightBrown),
            "red_background" | "light_red" => Ok(Color::LightRed),
            "orange_background" | "light_orange" => Ok(Color::LightOrange),
            "yellow_background" | "light_yellow" => Ok(Color::LightYellow),
            "green_background" | "light_green" => Ok(Color::LightGreen),
            "blue_background" | "light_blue" => Ok(Color::LightBlue),
            "purple_background" | "light_purple" => Ok(Color::LightPurple),
            "pink_background" | "light_pink" => Ok(Color::LightPink),
            _ => Err(ValidationError::InvalidColor(s.to_string())),
        }
    }
}

impl Color {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Color::Default => "default",
            Color::Gray => "gray",
            Color::Brown => "brown",
            Color::Red => "red",
            Color::Orange => "orange",
            Color::Yellow => "yellow",
            Color::Green => "green",
            Color::Blue => "blue",
            Color::Purple => "purple",
            Color::Pink => "pink",
            Color::LightGray => "gray_background",
            Color::LightBrown => "brown_background",
            Color::LightRed => "red_background",
            Color::LightOrange => "orange_background",
            Color::LightYellow => "yellow_background",
            Color::LightGreen => "green_background",
            Color::LightBlue => "blue_background",
            Color::LightPurple => "purple_background",
            Color::LightPink => "pink_background",
        }
    }

    /// Check if this is a background color
    #[allow(dead_code)]
    pub fn is_background(&self) -> bool {
        matches!(
            self,
            Color::LightGray
                | Color::LightBrown
                | Color::LightRed
                | Color::LightOrange
                | Color::LightYellow
                | Color::LightGreen
                | Color::LightBlue
                | Color::LightPurple
                | Color::LightPink
        )
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_color_parsing() {
        assert_eq!(Color::from_str("red").unwrap(), Color::Red);
        assert_eq!(
            Color::from_str("gray_background").unwrap(),
            Color::LightGray
        );
        assert_eq!(Color::from_str("light_gray").unwrap(), Color::LightGray);
        assert!(Color::from_str("invalid").is_err());
    }

    #[test]
    fn test_background_colors() {
        assert!(!Color::Red.is_background());
        assert!(Color::LightRed.is_background());
    }
}
