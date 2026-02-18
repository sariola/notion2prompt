// src/formatting/rich_text/annotations.rs
//! Annotation processing for rich text.
//!
//! This module handles the application of text annotations (styling)
//! in a structured and composable way.

use super::types::{TextColor, TextStyle};
use crate::types::Annotations;

/// Converts model annotations to text style.
pub fn annotations_to_style(annotations: &Annotations) -> TextStyle {
    TextStyle {
        bold: annotations.bold,
        italic: annotations.italic,
        strikethrough: annotations.strikethrough,
        underline: annotations.underline,
        code: annotations.code,
        color: parse_color(annotations.color.as_str()),
        link: None, // Links are handled separately
    }
}

/// Parses a color string into a TextColor.
fn parse_color(color: &str) -> TextColor {
    match color {
        "default" => TextColor::Default,
        "gray" => TextColor::Gray,
        "brown" => TextColor::Brown,
        "orange" => TextColor::Orange,
        "yellow" => TextColor::Yellow,
        "green" => TextColor::Green,
        "blue" => TextColor::Blue,
        "purple" => TextColor::Purple,
        "pink" => TextColor::Pink,
        "red" => TextColor::Red,
        "gray_background" => TextColor::GrayBackground,
        "brown_background" => TextColor::BrownBackground,
        "orange_background" => TextColor::OrangeBackground,
        "yellow_background" => TextColor::YellowBackground,
        "green_background" => TextColor::GreenBackground,
        "blue_background" => TextColor::BlueBackground,
        "purple_background" => TextColor::PurpleBackground,
        "pink_background" => TextColor::PinkBackground,
        "red_background" => TextColor::RedBackground,
        _ => {
            log::debug!("Unknown color: {}", color);
            TextColor::Default
        }
    }
}

/// Renderer for text styles to Markdown.
pub struct MarkdownStyleRenderer;

impl MarkdownStyleRenderer {
    /// Applies styles to text content for Markdown output.
    pub fn apply_styles(content: &str, style: &TextStyle) -> String {
        let mut result = content.to_string();

        // Apply code style first (it affects how other styles are applied)
        if style.code {
            result = format!("`{}`", result);
        }

        // Apply other text decorations
        if style.strikethrough {
            result = format!("~~{}~~", result);
        }

        if style.bold {
            result = format!("**{}**", result);
        }

        if style.italic {
            result = format!("*{}*", result);
        }

        // Underline requires HTML
        if style.underline {
            result = format!("<u>{}</u>", result);
        }

        // Apply link if present
        if let Some(url) = &style.link {
            result = format!("[{}]({})", result, url.as_str());
        }

        // Color requires HTML (optional, often skipped for pure Markdown)
        if style.color != TextColor::Default {
            // Could add HTML spans here if needed
            // For now, we ignore color in pure Markdown
        }

        result
    }

    /// Checks if content already looks like a Markdown link.
    pub fn is_markdown_link(content: &str) -> bool {
        content.starts_with('[') && content.contains("](") && content.ends_with(')')
    }
}

/// Renderer for text styles to HTML.
#[allow(dead_code)]
pub struct HtmlStyleRenderer;

impl HtmlStyleRenderer {
    /// Applies styles to text content for HTML output.
    #[allow(dead_code)]
    pub fn apply_styles(content: &str, style: &TextStyle, escape_content: bool) -> String {
        let escaped = if escape_content {
            html_escape(content)
        } else {
            content.to_string()
        };

        let mut result = escaped;

        // Apply styles using HTML tags
        if style.code {
            result = format!("<code>{}</code>", result);
        }

        if style.strikethrough {
            result = format!("<s>{}</s>", result);
        }

        if style.bold {
            result = format!("<strong>{}</strong>", result);
        }

        if style.italic {
            result = format!("<em>{}</em>", result);
        }

        if style.underline {
            result = format!("<u>{}</u>", result);
        }

        // Apply color if not default
        if style.color != TextColor::Default {
            let color_class = color_to_css_class(style.color);
            result = format!("<span class=\"{}\">{}</span>", color_class, result);
        }

        // Apply link
        if let Some(url) = &style.link {
            result = format!("<a href=\"{}\">{}</a>", html_escape(url.as_str()), result);
        }

        result
    }
}

/// Converts a TextColor to a CSS class name.
#[allow(dead_code)]
fn color_to_css_class(color: TextColor) -> &'static str {
    match color {
        TextColor::Default => "text-default",
        TextColor::Gray => "text-gray",
        TextColor::Brown => "text-brown",
        TextColor::Orange => "text-orange",
        TextColor::Yellow => "text-yellow",
        TextColor::Green => "text-green",
        TextColor::Blue => "text-blue",
        TextColor::Purple => "text-purple",
        TextColor::Pink => "text-pink",
        TextColor::Red => "text-red",
        TextColor::GrayBackground => "bg-gray",
        TextColor::BrownBackground => "bg-brown",
        TextColor::OrangeBackground => "bg-orange",
        TextColor::YellowBackground => "bg-yellow",
        TextColor::GreenBackground => "bg-green",
        TextColor::BlueBackground => "bg-blue",
        TextColor::PurpleBackground => "bg-purple",
        TextColor::PinkBackground => "bg-pink",
        TextColor::RedBackground => "bg-red",
    }
}

/// Basic HTML escaping.
#[allow(dead_code)]
fn html_escape(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_style_application() {
        let style = TextStyle {
            bold: true,
            italic: true,
            ..Default::default()
        };

        let result = MarkdownStyleRenderer::apply_styles("test", &style);
        assert_eq!(result, "***test***");
    }

    #[test]
    fn test_code_style_priority() {
        let style = TextStyle {
            code: true,
            bold: true,
            ..Default::default()
        };

        let result = MarkdownStyleRenderer::apply_styles("test", &style);
        assert_eq!(result, "**`test`**");
    }

    #[test]
    fn test_link_detection() {
        assert!(MarkdownStyleRenderer::is_markdown_link("[text](url)"));
        assert!(!MarkdownStyleRenderer::is_markdown_link("normal text"));
        assert!(!MarkdownStyleRenderer::is_markdown_link("[incomplete"));
    }
}
