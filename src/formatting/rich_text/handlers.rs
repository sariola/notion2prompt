// src/formatting/rich_text/handlers.rs
//! Handlers for different mention types in rich text.
//!
//! This module provides specialized handlers for processing different
//! types of mentions, keeping the logic modular and testable.

use super::types::*;
use crate::types::DateValue;
use crate::types::{
    DatabaseReference, LinkPreviewReference, MentionData, MentionType, NotionId, PageReference,
    PartialUser,
};

/// Trait for handling mention types.
#[allow(dead_code)]
pub trait MentionHandler {
    /// Handles a mention and returns formatted content.
    fn handle(&self, mention: &MentionData, plain_text: &str) -> MentionContent;
}

/// Registry of mention handlers.
pub struct MentionHandlerRegistry {
    user_handler: UserMentionHandler,
    page_handler: PageMentionHandler,
    database_handler: DatabaseMentionHandler,
    date_handler: DateMentionHandler,
    link_handler: LinkMentionHandler,
}

impl MentionHandlerRegistry {
    /// Creates a new mention handler registry.
    pub fn new() -> Self {
        Self {
            user_handler: UserMentionHandler,
            page_handler: PageMentionHandler,
            database_handler: DatabaseMentionHandler,
            date_handler: DateMentionHandler,
            link_handler: LinkMentionHandler,
        }
    }

    /// Handles a mention based on its type.
    pub fn handle(&self, mention: &MentionData, plain_text: &str) -> MentionContent {
        match &mention.mention_type {
            MentionType::User { user } => self.user_handler.handle_user(user),
            MentionType::Page { page } => self.page_handler.handle_page(page, plain_text),
            MentionType::Database { database } => {
                self.database_handler.handle_database(database, plain_text)
            }
            MentionType::Date { date } => self.date_handler.handle_date(date),
            MentionType::LinkPreview { link_preview } => self
                .link_handler
                .handle_link_preview(link_preview, plain_text),
            MentionType::LinkMention { url } => {
                self.link_handler.handle_link_mention(url, plain_text)
            }
        }
    }
}

/// Handler for user mentions.
struct UserMentionHandler;

impl UserMentionHandler {
    fn handle_user(&self, user: &PartialUser) -> MentionContent {
        MentionContent::User {
            id: user.id.clone(),
            name: user.name.as_deref().unwrap_or(&user.id).to_string(),
        }
    }
}

/// Handler for page mentions.
struct PageMentionHandler;

impl PageMentionHandler {
    fn handle_page(&self, page: &PageReference, plain_text: &str) -> MentionContent {
        let title = if plain_text.is_empty() {
            "Page"
        } else {
            plain_text
        };
        MentionContent::Page {
            id: page.id.clone(),
            title: title.to_string(),
        }
    }
}

/// Handler for database mentions.
struct DatabaseMentionHandler;

impl DatabaseMentionHandler {
    fn handle_database(&self, database: &DatabaseReference, plain_text: &str) -> MentionContent {
        let title = if plain_text.is_empty() {
            "Database"
        } else {
            plain_text
        };
        MentionContent::Database {
            id: database.id.clone(),
            title: title.to_string(),
        }
    }
}

/// Handler for date mentions.
struct DateMentionHandler;

impl DateMentionHandler {
    fn handle_date(&self, date: &DateValue) -> MentionContent {
        MentionContent::Date {
            start: date.start.to_string(),
            end: date.end.map(|e| e.to_string()),
        }
    }
}

/// Handler for link mentions.
struct LinkMentionHandler;

impl LinkMentionHandler {
    fn handle_link_preview(
        &self,
        link_preview: &LinkPreviewReference,
        plain_text: &str,
    ) -> MentionContent {
        self.create_link_mention(&link_preview.url, plain_text)
    }

    fn handle_link_mention(&self, url: &str, plain_text: &str) -> MentionContent {
        self.create_link_mention(url, plain_text)
    }

    fn create_link_mention(&self, url: &str, plain_text: &str) -> MentionContent {
        match ValidatedUrl::parse(url) {
            Ok(validated_url) => MentionContent::Link {
                url: validated_url,
                text: plain_text.to_string(),
            },
            Err(_) => {
                log::warn!("Invalid URL in mention: {}", url);
                // Create a safe fallback URL that displays the original text
                MentionContent::Link {
                    url: ValidatedUrl::parse("about:blank")
                        .expect("about:blank should always be a valid URL"),
                    text: format!("{} (invalid URL: {})", plain_text, url),
                }
            }
        }
    }
}

/// Checks if text indicates a database reference.
pub fn is_database_reference(text: &str, url: &ValidatedUrl) -> bool {
    if !url.is_notion_url() {
        return false;
    }

    let db_indicators = ["database", "db", "table", "key highlights"];
    let text_lower = text.to_lowercase();

    db_indicators
        .iter()
        .any(|indicator| text_lower.contains(indicator))
}

/// Extracts a Notion ID from a URL if possible.
pub fn extract_notion_id(url: &ValidatedUrl) -> Option<NotionId> {
    if url.is_notion_url() {
        NotionId::parse(url.as_str()).ok()
    } else {
        None
    }
}
